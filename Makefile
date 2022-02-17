build:
	@docker run -it --user "$(id -u)":"$(id -g)" -v $(PWD):/usr/src/myapp -w /usr/src/myapp rust cargo b

test:
	./test.sh
	./test-driver.sh
