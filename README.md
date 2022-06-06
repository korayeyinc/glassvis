# ImgDiff

GTK3 application for displaying image diffs between images.


## Features
* Supports encoding/decoding of 'BMP', 'GIF', 'JPG', 'JPEG', 'PNG', 'PNM', 'TGA', 'TIFF', 'WEBP' images.
* Supports setting defect significance.
* Supports choosing marker color.


## Requirements

* GTK+3 (>= v3.22) (https://www.gtk.org/)


## Installation

Debian/Ubuntu:
```sh
apt install libgtk-3-0
```


## TODO

* Detect image format from memory block instead of file extension.
* Write tests.


## References

* http://www.imagemagick.org/Usage/compare/#statistics
* https://developer.gimp.org/api/2.0/gdk-pixbuf/gdk-pixbuf-scaling.html
* https://www.smashingmagazine.com/2015/06/efficient-image-resizing-with-imagemagick/
