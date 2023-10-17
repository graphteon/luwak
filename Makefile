VERSION=`cat Cargo.toml|grep "version ="|head -n 1|sed 's|version = ||g'|sed 's|"||g'`
IMAGE="orcinus/luwak"

all: subdirs release

subdirs:
	 $(MAKE) -C luwak

build: subdirs
	cargo build

release: subdirs
	@cargo fmt
	@cargo build --release

install: release
	@cp -rf target/release/luwak $(HOME)/.luwak/bin

dockerx: subdirs
	# docker buildx create --name luwak --use
	docker buildx build --push --platform linux/arm/v7,linux/arm64,linux/amd64 --tag orcinus/luwak:$(VERSION) .


docker: subdirs
	@echo "add luwak to docker image"
	docker build -t $(IMAGE):$(VERSION) .; \
	docker tag $(IMAGE):$(VERSION) $(IMAGE):latest; \
	docker push $(IMAGE):$(VERSION); \
	docker push $(IMAGE):latest; \
	docker rmi $(IMAGE):$(VERSION); \
	docker rmi $(IMAGE):latest;
	rm -rf build

# docker: dockerbuild dockerbin

# dockerbuild:
# 	rm -rf build
# 	for arch in "amd64" "arm32v7" "arm64v8";do \
# 		echo "### docker build: builder image $$arch"; \
# 		mkdir build; \
# 		docker build -t builder -f Dockerfile.build --build-arg ARCH=$$arch .; \
# 		echo "### extract odorous"; \
# 		docker create --name tmp builder; \
# 		docker cp tmp:/src/target/release/luwak build/luwak-$$arch; \
# 		docker rm -vf tmp; \
# 		docker rmi builder; \
# 	done

# dockerbin:
# 	@echo "add luwak to docker image"
# 	for arch in "amd64" "arm32v7" "arm64v8";do \
# 		docker build -t $(IMAGE):$(VERSION) --build-arg ARCH=$$arch .; \
# 		docker tag $(IMAGE):$(VERSION) $(IMAGE):latest; \
# 		docker push $(IMAGE):$(VERSION); \
# 		docker push $(IMAGE):latest; \
# 		docker rmi $(IMAGE):$(VERSION); \
# 		docker rmi $(IMAGE):latest; \
# 	done
# 	rm -rf build