
BINDIR= /usr/local/bin

hyperstencil: 
	cargo build --release

install:
	cp target/release/hyperstencil $(BINDIR)
        
clean:
	rm -rf target
