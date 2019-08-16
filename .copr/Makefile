CARGO := $(shell if which cargo 2>/dev/null; then true; else echo cargo; fi)
BUILDROOT:=$(shell [ -d "/build" ] && echo "/build" || echo ".")
TMPDIR:=$(shell mktemp -d)

$(CARGO):
	yum install -y cargo

srpm: $(CARGO)
	mkdir -p ${TMPDIR}/_topdir/SOURCES
	mkdir -p ${TMPDIR}/_topdir/SPECS
	mkdir -p ${TMPDIR}/release/lustre_collector
	rm -rf ${BUILDROOT}/_topdir
	cargo build --release
	cp target/release/lustre_collector ${TMPDIR}/release/lustre_collector
	tar -czvf ${TMPDIR}/_topdir/SOURCES/lustre_collector.tar.gz -C ${TMPDIR}/release/lustre_collector .
	cp lustre_collector.spec ${TMPDIR}/_topdir/SPECS/
	rpmbuild -bs -D "_topdir ${TMPDIR}/_topdir" ${TMPDIR}/_topdir/SPECS/lustre_collector.spec
	cp -rf ${TMPDIR}/_topdir ${BUILDROOT}/
	cp -f _topdir/SRPMS/*.rpm $(outdir)
