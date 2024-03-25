## Performance considerations

Since GPU rendering of a large count of DOM counts can significantly slow down devices, do NOT apply CSS directives that move the element composition onto the GPU on a large amount of DOM nodes unless absolutely necessary.

Historically panes have all have had a 1px background blur, which caused massive lags and graphical glitches, since panes are used everywhere.

!!! success "Do"
    ```css
    .my-popup { /* filter / 3D transform / etc. */ }
    ```

    Popups are only visible for a short amount of time, usually via user-interaction, and usually only exist once or twice at once. Blurs and such can help distinguish content and give improved looks.

!!! failure "Don't"
    ```css
    .pane { /* filter / 3D transform / etc. */ }
    ```
    
    A pane is always visible and there are potentially a lot of them. Blurs are unnecessary since there is usually no content behind them other than the background anyway. To help keep text readable with the animated matrix background behind it, instead consider increasing the background opacity to make it more opaque and adjusting the colors accordingly.

    The built-in pane classes already help do all of this.

!!! failure "Don't"
    ```css
        backdrop-filter: blur(0.1em);
    ```

    This blur is likely equal to 1-2px of blur. An amount that is unlikely to give any readability improvements, nor look very good. Consider a higher background opacity instead and remove the blur entirely.

!!! failure "Don't"
    ```css
    button { backdrop-filter: blur(40px); }
    ```

    A button is usually a small element and this applies a massive blur, which is likely going to just result in a single color. Instead use a color picker to pick the color and just use it as opaque background, possibly with a little opacity for a nicer look. Gradients may also help achieve this effect on larger elements.

### Directives that need special attention

!!! warning "Blurring"
    ```css
        backdrop-filter: blur(...);
        filter: blur(...);
    ```

    Blurs can help differentiate foreground content from background content. Only use them in combination with user-dismissible popups.

    Never use blurs on content that is permanently visible without user interaction or can't be removed from the render DOM.

    More opaque background colors as well as gradients and box shadows should be used instead.

!!! info "filter: drop-shadow"
    ```css
        filter: drop-shadow(...);
    ```

    Drop shadow is a more accurate shadow than a box shadow and can accurately trace the outline of the element / image. Only use this when the shape is not a (rounded) rectangle, since it's not as well optimized as a `box-shadow` and doesn't offer as much control.

!!! info "Animating CSS --variables"
    ```css
    @keyframes {
        ... { --my-variable: ...; }
    }
    ```

    Animating CSS variables is only supported on chromium based browsers and when a `@property` for them is defined. On Firefox the animation will be skipped / just switch from one value to the other in the half-way point of the animation.

    Avoid if possible, only use for things where a missing animation wouldn't cause it to look too bad.

!!! info "Animating filters"
    ```css
        transition: filter ...;
        transition: all ...;
        will-change: filter;
    ```

    Having filters potentially animated can cause the browser to possibly render the element on the GPU. Avoid applying this to many elements at once since the behavior can be unpredictable.

    This will usually not cause issues, but should be considered as a last resort depending on the target result. Filter animations can very easily become laggy and certain stuff like color manipulation can often be solved by adjusting colors.

!!! info "Transforms"
    ```css
        transform: scale(...) rotate(...);
        scale: ...;
        rotate: ...;
        animation: my-transform-animation ...;
    ```

    Scaling and rotation can also move element rendering onto the GPU as well as make text potentially blurry, especially when animated.

    Consider the impacts and try to use them on short interaction animations or for fixed transforms. Avoid too many active transform animations at once on a page.
