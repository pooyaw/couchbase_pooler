VERSION=2.10.3
binding:
	@echo Generating binding for version "${VERSION}"
	wget -N https://github.com/couchbase/libcouchbase/archive/${VERSION}.tar.gz
	tar -xvzpf ${VERSION}.tar.gz
	echo "#include \"libcouchbase-${VERSION}/include/libcouchbase/couchbase.h\"\n\
		#include \"libcouchbase-${VERSION}/include/libcouchbase/n1ql.h\"\n\
		#include \"libcouchbase-${VERSION}/include/libcouchbase/cbft.h\"\n\
		#include \"libcouchbase-${VERSION}/include/libcouchbase/views.h\"" > headers-${VERSION}.h 	
	bindgen --verbose --no-doc-comments headers-${VERSION}.h -- -I libcouchbase-${VERSION}/include/libcouchbase/ > src/bindings-${VERSION}.rs
	rm -rf ${VERSION}.tar.gz
	#rm headers-${VERSION}.h
