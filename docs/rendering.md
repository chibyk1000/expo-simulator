# Rendering & Layout Pipeline

## Overview

The rendering pipeline transforms the abstract React Native component tree into pixels on screen.

```text
React Native Component Tree (View, Text, TextInput, Pressable, ScrollView)
                            │
                            ▼
           Yoga Flexbox Tree (simulator-renderer)
                            │
                            ▼
        Computed Layout Metrics (x, y, width, height)
                            │
                            ▼
        Scene Graph & Hit-Testing Tree (simulator-input)
                            │
                            ▼
         2D Vector Renderer (tiny-skia + cosmic-text)
                            │
                            ▼
         Desktop Window Framebuffer (softbuffer / winit)
```

---

## Layout with Yoga

React Native relies on Meta's **Yoga** engine for CSS Flexbox calculations.
In `simulator-renderer`, each component node corresponds to a `yoga::Node`.

Supported styles in Phase 1:
- `flex`, `flexGrow`, `flexShrink`, `flexBasis`
- `flexDirection` (`column`, `row`, `column-reverse`, `row-reverse`)
- `justifyContent` (`flex-start`, `center`, `flex-end`, `space-between`, `space-around`, `space-evenly`)
- `alignItems`, `alignSelf` (`flex-start`, `center`, `flex-end`, `stretch`, `baseline`)
- `padding`, `paddingHorizontal`, `paddingVertical`, `paddingTop`, `paddingBottom`, `paddingLeft`, `paddingRight`
- `margin`, `marginHorizontal`, `marginVertical`, `marginTop`, `marginBottom`, `marginLeft`, `marginRight`
- `width`, `height`, `minWidth`, `maxWidth`, `minHeight`, `maxHeight`
- `position` (`relative`, `absolute`), `top`, `bottom`, `left`, `right`

---

## 2D Drawing with tiny-skia & cosmic-text

Visual representation uses high-performance 2D drawing:

* **Backgrounds**: Hex/RGB colors, opacity blending.
* **Borders & Corners**: `borderRadius`, individual corner radiuses, `borderColor`, `borderWidth`.
* **Clipping**: `overflow: 'hidden'`, rounded corner clipping paths.
* **Text**: Multi-line paragraph formatting, font size, font weight, line height, color, and word wrapping handled by `cosmic-text`.
* **Scrolling**: `ScrollView` calculates content height against viewport height and applies scroll translations.
