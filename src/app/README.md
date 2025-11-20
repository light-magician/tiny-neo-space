
Rendering VS Drawing Loops
---
- The render loop is the main game loop that runs every frame. In this case, it's the loop in the run function.
- A draw call is a specific instruction to the GPU to draw something. In Macroquad, functions like draw_texture result in draw calls.
- The render loop runs every frame, updating game state and deciding what to draw.
- Draw calls are the actual rendering instructions within that loop.

### Optimizing Rendering + Drawing:
- By rendering to textures and only updating those textures when necessary, we reduce the number of draw calls per frame.
- Instead of redrawing all UI elements and strokes every frame, we're now just drawing two textures most frames.

### Performance Implications:

This approach significantly reduces GPU work for static scenes.
It's particularly effective for complex UIs or drawings that don't change every frame.


### Trade-offs:

This method uses more memory (for storing textures) but less GPU time.
It's great for static content but might need refinement for highly dynamic scenes.


### Further Optimizations:

You could implement partial updates to these textures for even better performance in some scenarios.
For very large canvases, you might want to implement a tiling system to render only visible parts.