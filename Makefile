VERSION=`cat Cargo.toml|grep "version ="|head -n 1|sed 's|version = ||g'|sed 's|"||g'`
IMAGE="orcinus/luwak"

all: build

build: cargo build

release: cargo build --release

docker: dockerbuild dockerbin

dockerbuild:
	rm -rf build
	for arch in "amd64" "arm32v7" "arm64v8";do \
		echo "### docker build: builder image $$arch"; \
		mkdir build; \
		docker build -t builder -f Dockerfile.build --build-arg ARCH=$$arch .; \
		echo "### extract odorous"; \
		docker create --name tmp builder; \
		docker cp tmp:/src/target/release/luwak build/luwak-$$arch; \
		docker rm -vf tmp; \
		docker rmi builder; \
	done

dockerbin:
	@echo "add luwak to docker image"
	for arch in "amd64" "arm32v7" "arm64v8";do \
		docker build -t $$arch/$(IMAGE):$(VERSION) --build-arg ARCH=$$arch .; \
		docker tag $$arch/$(IMAGE):$(VERSION) $(IMAGE):latest; \
		docker push $$arch/$(IMAGE):$(VERSION); \
		docker push $$arch/$(IMAGE):latest; \
		docker rmi $$arch/$(IMAGE):$(VERSION); \
		docker rmi $$arch/$(IMAGE):latest; \
	done
	rm -rf build