# Images

Types related to image reading and writing.


## OmfImageMode

```c
typedef enum { ... } OmfImageMode;
```

Describes what channels the image data has.

### Options

OMF_IMAGE_MODE_GRAY = 1
: Grayscale. One channel.

OMF_IMAGE_MODE_GRAY_ALPHA = 2
: Grayscale with alpha. Two channels.

OMF_IMAGE_MODE_RGB = 3
: Red, green, and blue. Three channels.

OMF_IMAGE_MODE_RGBA = 4
: Red, green, blue, and alpha. Four channels.


## OmfImageData

```c
typedef struct {
    uint32_t width;
    uint32_t height;
    OmfImageMode mode;
    const uint8_t *uint8;
    const uint16_t *uint16;
} OmfImageData;
```

The type returned when reading image data from the file.
The image can have 8 or 16 bits-per-channel and be grayscale, grayscale-alpha, RGB, or RGBA channels.

### Fields

width: `uint32_t`
: The image width in pixels.

height: `uint32_t`
: The image height in pixels.

mode: [`OmfImageMode`](#OmfImageMode)
: What channels the image data has.

uint8: `const uint8_t*`
: 

uint16: `const uint16_t*`
: Pixel data in 8 or 16 bits per channel.
Exactly one will be non-null.
There is no padding or row alignment.

### Methods

#### omf_image_data_free

```c
bool omf_image_data_free(OmfImageData *data);
```

Call to free an `OmfImageData` pointer when you are finished with it. Returns false on error.
