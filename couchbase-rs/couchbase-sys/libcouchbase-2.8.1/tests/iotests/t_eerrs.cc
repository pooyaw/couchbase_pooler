#include "config.h"
#include "iotests.h"
#include "internal.h"

class EerrsUnitTest : public MockUnitTest {
protected:
    virtual void createEerrConnection(HandleWrap& hw, lcb_t& instance) {
        MockEnvironment::getInstance()->createConnection(hw, instance);
        ASSERT_EQ(LCB_SUCCESS, lcb_connect(instance));
        lcb_wait(instance);
        ASSERT_EQ(LCB_SUCCESS, lcb_get_bootstrap_status(instance));
    }

    void enableEnhancedErrors() {
        MockEnvironment::getInstance()->setEnhancedErrors(true);
    }

    void disableEnhancedErrors() {
        MockEnvironment::getInstance()->setEnhancedErrors(false);
    }

    void checkRetryVerify(uint16_t errcode);

    void TearDown() {
        MockOpFailClearCommand clearCmd(MockEnvironment::getInstance()->getNumNodes());
        doMockTxn(clearCmd);
        MockUnitTest::TearDown();
    }
};

struct EerrsCookie {
    lcb_error_t rc;
    bool called;
    char *err_ref;
    char *err_ctx;

    void reset() {
        rc = LCB_SUCCESS;
        called = false;
        free(err_ref);
        err_ref = NULL;
        free(err_ctx);
        err_ctx = NULL;
    }
    EerrsCookie() : rc(LCB_SUCCESS), called(false), err_ref(NULL), err_ctx(NULL) {
    }

    ~EerrsCookie() {
        free(err_ref);
        free(err_ctx);
    }
};

extern "C" {
static void opcb(lcb_t, int cbtype, const lcb_RESPBASE* rb) {
    EerrsCookie *cookie = reinterpret_cast<EerrsCookie*>(rb->cookie);
    cookie->called = true;
    cookie->rc = rb->rc;

    const char *ref = lcb_resp_get_error_ref(cbtype, rb);
    if (ref != NULL) {
        cookie->err_ref = strdup(ref);
    }

    const char *ctx = lcb_resp_get_error_context(cbtype, rb);
    if (ctx != NULL) {
        cookie->err_ctx = strdup(ctx);
    }
}
}

TEST_F(EerrsUnitTest, testInCallbackWhenEnabled) {
    SKIP_UNLESS_MOCK();
    HandleWrap hw;
    lcb_t instance;

    enableEnhancedErrors();
    createEerrConnection(hw, instance);
    lcb_install_callback3(instance, LCB_CALLBACK_DEFAULT, opcb);

    EerrsCookie cookie;

    std::string key("hello");
    lcb_CMDGET cmd = { 0 };
    LCB_CMD_SET_KEY(&cmd, key.c_str(), key.size());
    cmd.lock = 10;
    lcb_get3(instance, &cookie, &cmd);

    lcb_wait(instance);
    ASSERT_TRUE(cookie.called);
    ASSERT_EQ(LCB_KEY_ENOENT, cookie.rc);
    ASSERT_NE((char *)NULL, cookie.err_ref);
    ASSERT_EQ(36, strlen(cookie.err_ref)); // java.util.UUID generates 36-bytes long strings
    ASSERT_STREQ("Failed to lookup item", cookie.err_ctx);
}


TEST_F(EerrsUnitTest, testInCallbackWhenDisabled) {
    SKIP_UNLESS_MOCK();
    HandleWrap hw;
    lcb_t instance;

    disableEnhancedErrors();
    createEerrConnection(hw, instance);
    lcb_install_callback3(instance, LCB_CALLBACK_DEFAULT, opcb);

    EerrsCookie cookie;

    std::string key("hello");
    lcb_CMDGET cmd = { 0 };
    LCB_CMD_SET_KEY(&cmd, key.c_str(), key.size());
    cmd.lock = 10;
    lcb_get3(instance, &cookie, &cmd);

    lcb_wait(instance);
    ASSERT_TRUE(cookie.called);
    ASSERT_EQ((char *)NULL, cookie.err_ref);
    ASSERT_EQ((char *)NULL, cookie.err_ctx);
}
