VERSION=`cat Cargo.toml|grep "version ="|head -n 1|sed 's|version = ||g'|sed 's|"||g'`
IMAGE="orcinus/luwak"

all: build

build: cargo build

release: cargo build --release

docker: dockerbuild dockerbin

dockerbuild:
	rm -rf build
	@echo "### docker build: builder image"
	mkdir build
	docker build -t builder -f Dockerfile.build .
	@echo "### extract odorous"
	docker create --name tmp builder
	docker cp tmp:/src/target/release/luwak build/luwak
	docker rm -vf tmp
	docker rmi builder

dockerbin:
	@echo "add luwak to docker image"
	docker build -t $(IMAGE):$(VERSION) .
	docker tag $(IMAGE):$(VERSION) $(IMAGE):latest
	docker push $(IMAGE):$(VERSION)
	docker push $(IMAGE):latest
	docker rmi $(IMAGE):$(VERSION)
	docker rmi $(IMAGE):latest
	rm -rf build