# Hyperstencil

Hyperstencil is a command line utility that lets you convert between JPEG or PNG
images, and raw (headerless) files with an arbitrary number of _layers_ per
channel. It does this by subdividing each color channel by intensity, with lower
layers containing only darker pixel values, and higher layers containing
brighter ones, like a set of stencils. This is a very inefficient image format,
but can be interesting for sonification purposes. For instance, a 4-layer RGB
raw image can be imported as a 12-channel audio file in Audacity, manipulated,
and exported for re-assembly.

## Building/Installation

Make sure you have a recent version of
[Rust](https://www.rust-lang.org/en-US/install.html), and run:

```
make && make install
```

in the top-level project directory. By default the binary will be installed in
/usr/local/bin, but this can be customized in the Makefile.


## Usage

Encoding:

```
USAGE:

hyperstencil encode <INPUT> <OUTPUT> --layers <layers>
```

Decoding:

```
USAGE:

hyperstencil decode <INPUT> <OUTPUT> --height <height> --layers <layers> --width <width>
```

Examples:

```
hyperstencil encode -l4 input.png output.raw
hyperstencil encode --layers 4 input.png output.raw

hyperstencil decode -l4 -w640 -h480 input.raw output.png
hyperstencil decode --layers 4 --width 640 --height 480 input.raw output.png
```

## A few notes on behavior

If your raw file and the dimensions or layers supplied in decoding don't match,
Hyperstencil will do its best to re-assemble _something_, but it will probably
look quite strange. Depending on your tastes in glitch art, this could actually
be to your liking. Layer values of 0 are coerced to 1, which is just a regular
old image. Keep in mind that for an RGB source image every additional layer
increases the raw file size by a factor of 3. Otherwise don't be afraid to get
weird with it.
