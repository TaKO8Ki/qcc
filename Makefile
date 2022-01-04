build:
	@docker run -it --user "$(id -u)":"$(id -g)" -v $(PWD):/usr/src/myapp -w /usr/src/myapp rust cargo b

test:
	@docker run -it --user "$(id -u)":"$(id -g)" -v $(PWD):/usr/src/myapp -w /usr/src/myapp rust ./test.sh
