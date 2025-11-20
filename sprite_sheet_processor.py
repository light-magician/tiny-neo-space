#!/usr/bin/env python3
"""
Sprite Sheet Background Removal Tool

This script processes a sprite sheet by:
1. Detecting white grid lines that divide sprites
2. Extracting individual cells from the grid
3. Analyzing each cell to find the actual sprite vs colored background
4. Removing the colored background from each sprite
5. Organizing sprites into row folders
"""

import cv2
import numpy as np
from pathlib import Path
import argparse


def detect_white_grid_lines(img, white_threshold=240):
    """
    Detect white grid lines using morphological operations.

    Args:
        img: OpenCV image (BGR format)
        white_threshold: Threshold for detecting white pixels (240-255)

    Returns:
        Tuple of (grid_mask, horizontal_lines, vertical_lines)
    """
    print("Detecting white grid lines...")
    gray = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)

    # Threshold to isolate white lines
    _, thresh = cv2.threshold(gray, white_threshold, 255, cv2.THRESH_BINARY)

    # Detect HORIZONTAL lines using morphological operations
    # Kernel: wide and short to detect horizontal structures
    horizontal_kernel = cv2.getStructuringElement(cv2.MORPH_RECT, (40, 1))
    horizontal_lines = cv2.morphologyEx(thresh, cv2.MORPH_OPEN, horizontal_kernel, iterations=2)

    # Detect VERTICAL lines
    # Kernel: tall and narrow to detect vertical structures
    vertical_kernel = cv2.getStructuringElement(cv2.MORPH_RECT, (1, 40))
    vertical_lines = cv2.morphologyEx(thresh, cv2.MORPH_OPEN, vertical_kernel, iterations=2)

    # Combine horizontal and vertical lines to get complete grid
    grid_mask = cv2.add(horizontal_lines, vertical_lines)

    # Count lines detected
    h_count = np.count_nonzero(np.any(horizontal_lines > 0, axis=1))
    v_count = np.count_nonzero(np.any(vertical_lines > 0, axis=0))
    print(f"  Found ~{h_count} horizontal lines and ~{v_count} vertical lines")

    return grid_mask, horizontal_lines, vertical_lines


def extract_cells_from_grid(img, grid_mask, min_cell_size=10):
    """
    Extract individual cells from the grid using contour detection.

    Args:
        img: Original image
        grid_mask: Binary mask of grid lines (white lines = 255)
        min_cell_size: Minimum width/height for valid cells

    Returns:
        List of dicts with 'image', 'bbox' (x, y, w, h), and 'position' (row, col)
    """
    print("Extracting cells from grid...")

    # Invert grid mask so cells become white, grid lines become black
    inverted = cv2.bitwise_not(grid_mask)

    # Find contours - each contour should be one cell
    contours, _ = cv2.findContours(inverted, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)

    cells = []
    for contour in contours:
        x, y, w, h = cv2.boundingRect(contour)

        # Filter out tiny cells (noise or grid line artifacts)
        if w >= min_cell_size and h >= min_cell_size:
            cell_img = img[y:y+h, x:x+w].copy()
            cells.append({
                'image': cell_img,
                'bbox': (x, y, w, h)
            })

    # Sort cells by position (top to bottom, left to right)
    cells.sort(key=lambda c: (c['bbox'][1], c['bbox'][0]))

    # Assign row/column positions
    if cells:
        # Group into rows based on y-coordinate (with some tolerance)
        row_tolerance = 5
        current_row = 0
        current_y = cells[0]['bbox'][1]
        col_in_row = 0

        for cell in cells:
            y = cell['bbox'][1]

            # Check if we've moved to a new row
            if abs(y - current_y) > row_tolerance:
                current_row += 1
                current_y = y
                col_in_row = 0

            cell['position'] = (current_row, col_in_row)
            col_in_row += 1

    print(f"  Extracted {len(cells)} cells")
    return cells


def find_cell_background_color(cell_img):
    """
    Find the dominant background color in a cell using color frequency analysis.

    Args:
        cell_img: Cell image (BGR format)

    Returns:
        Background color as BGR tuple
    """
    # Sample pixels from the cell's corners/edges (background is usually at edges)
    h, w = cell_img.shape[:2]

    # Sample a border around the edge
    border_size = max(2, min(h, w) // 10)

    edge_pixels = []
    # Top and bottom edges
    edge_pixels.append(cell_img[0:border_size, :].reshape(-1, 3))
    edge_pixels.append(cell_img[h-border_size:h, :].reshape(-1, 3))
    # Left and right edges
    edge_pixels.append(cell_img[:, 0:border_size].reshape(-1, 3))
    edge_pixels.append(cell_img[:, w-border_size:w].reshape(-1, 3))

    edge_pixels = np.vstack(edge_pixels)

    # Find most common color using bincount
    # Convert BGR triplets to single values for counting
    col_range = (256, 256, 256)
    pixel_indices = np.ravel_multi_index(edge_pixels.T, col_range)
    most_common_idx = np.bincount(pixel_indices).argmax()
    bg_color = np.unravel_index(most_common_idx, col_range)

    return tuple(bg_color)


def extract_sprite_from_cell_kmeans(cell_img, n_clusters=3):
    """
    Extract sprite from colored background using K-means clustering.

    Args:
        cell_img: Cell image containing sprite on colored background
        n_clusters: Number of color clusters (usually 2-4)

    Returns:
        RGBA image with transparent background
    """
    h, w = cell_img.shape[:2]

    # Reshape to list of pixels
    pixels = cell_img.reshape((-1, 3)).astype(np.float32)

    # Apply K-means clustering
    criteria = (cv2.TERM_CRITERIA_EPS + cv2.TERM_CRITERIA_MAX_ITER, 100, 0.2)
    _, labels, centers = cv2.kmeans(pixels, n_clusters, None, criteria, 10,
                                    cv2.KMEANS_RANDOM_CENTERS)

    labels = labels.reshape((h, w))

    # Identify background cluster by checking corners
    corner_labels = [
        labels[0, 0],           # top-left
        labels[0, w-1],         # top-right
        labels[h-1, 0],         # bottom-left
        labels[h-1, w-1]        # bottom-right
    ]

    # Most common corner label is likely the background
    bg_cluster = max(set(corner_labels), key=corner_labels.count)

    # Create mask: background = 0, sprite = 255
    sprite_mask = (labels != bg_cluster).astype(np.uint8) * 255

    # Smooth the mask slightly to handle anti-aliasing
    sprite_mask = cv2.GaussianBlur(sprite_mask, (3, 3), 0)

    # Create RGBA image
    sprite_rgba = cv2.cvtColor(cell_img, cv2.COLOR_BGR2BGRA)
    sprite_rgba[:, :, 3] = sprite_mask

    return sprite_rgba


def extract_sprite_from_cell_color_freq(cell_img, tolerance=40):
    """
    Extract sprite by removing the most frequent (background) color.

    Args:
        cell_img: Cell image
        tolerance: Color distance tolerance

    Returns:
        RGBA image with transparent background
    """
    # Find background color
    bg_color = find_cell_background_color(cell_img)

    # Calculate color distance from background
    color_diff = np.abs(cell_img.astype(np.float32) - np.array(bg_color))
    distance = np.sqrt(np.sum(color_diff ** 2, axis=2))

    # Create smooth alpha transition using sigmoid function
    # This preserves anti-aliasing on sprite edges
    alpha = 255 / (1 + np.exp(-(distance - tolerance) / 10))
    alpha = np.clip(alpha, 0, 255).astype(np.uint8)

    # Create RGBA image
    sprite_rgba = cv2.cvtColor(cell_img, cv2.COLOR_BGR2BGRA)
    sprite_rgba[:, :, 3] = alpha

    return sprite_rgba


def crop_to_content(sprite_rgba, padding=1):
    """
    Crop sprite to bounding box of non-transparent pixels.

    Args:
        sprite_rgba: RGBA image
        padding: Pixels of padding to add around sprite

    Returns:
        Cropped RGBA image
    """
    alpha = sprite_rgba[:, :, 3]
    coords = cv2.findNonZero(alpha)

    if coords is not None:
        x, y, w, h = cv2.boundingRect(coords)

        # Add padding
        h_max, w_max = sprite_rgba.shape[:2]
        x = max(0, x - padding)
        y = max(0, y - padding)
        w = min(w_max - x, w + 2*padding)
        h = min(h_max - y, h + 2*padding)

        return sprite_rgba[y:y+h, x:x+w]

    return sprite_rgba


def process_sprite_sheet(
    sheet_path,
    output_dir=None,
    method='kmeans',
    n_clusters=3,
    tolerance=40,
    white_threshold=240,
    crop=True,
    padding=1
):
    """
    Main processing function using the proper grid-based workflow.

    Args:
        sheet_path: Path to sprite sheet image
        output_dir: Output directory (defaults to sheet_name + "_processed")
        method: Extraction method ('kmeans' or 'color_freq')
        n_clusters: Number of clusters for kmeans (2-4 typical)
        tolerance: Color tolerance for color_freq method
        white_threshold: Threshold for detecting white grid lines (240-255)
        crop: Whether to crop sprites to content
        padding: Pixels of padding when cropping
    """
    sheet_path = Path(sheet_path)

    if not sheet_path.exists():
        print(f"Error: {sheet_path} does not exist")
        return

    # Setup output directory
    if output_dir is None:
        output_dir = Path(sheet_path.stem + "_processed")
    else:
        output_dir = Path(output_dir)

    output_dir.mkdir(exist_ok=True)
    print(f"\n{'='*60}")
    print(f"SPRITE SHEET PROCESSOR")
    print(f"{'='*60}")
    print(f"Input: {sheet_path}")
    print(f"Output: {output_dir}")
    print(f"Method: {method}")
    print(f"{'='*60}\n")

    # Load sprite sheet
    img = cv2.imread(str(sheet_path))

    if img is None:
        print(f"Error: Could not load image from {sheet_path}")
        return

    print(f"Image size: {img.shape[1]}x{img.shape[0]} pixels\n")

    # STEP 1: Detect white grid lines
    print("STEP 1: Detecting white grid lines")
    print("-" * 60)
    grid_mask, h_lines, v_lines = detect_white_grid_lines(img, white_threshold)

    # STEP 2: Extract cells from grid
    print("\nSTEP 2: Extracting cells from grid")
    print("-" * 60)
    cells = extract_cells_from_grid(img, grid_mask)

    if not cells:
        print("Error: No cells detected!")
        return

    # Organize cells by row
    rows = {}
    for cell in cells:
        row_num = cell['position'][0]
        if row_num not in rows:
            rows[row_num] = []
        rows[row_num].append(cell)

    print(f"  Organized into {len(rows)} rows")

    # STEP 3: Process each cell to extract sprite
    print("\nSTEP 3: Extracting sprites from colored backgrounds")
    print("-" * 60)

    total_sprites = 0

    for row_num in sorted(rows.keys()):
        row_cells = rows[row_num]
        row_dir = output_dir / f"row_{row_num:03d}"
        row_dir.mkdir(exist_ok=True)

        print(f"Row {row_num}: Processing {len(row_cells)} sprites...")

        for cell in row_cells:
            col_num = cell['position'][1]
            cell_img = cell['image']

            # Extract sprite using chosen method
            if method == 'kmeans':
                sprite_rgba = extract_sprite_from_cell_kmeans(cell_img, n_clusters)
            else:  # color_freq
                sprite_rgba = extract_sprite_from_cell_color_freq(cell_img, tolerance)

            # Crop to content if requested
            if crop:
                sprite_rgba = crop_to_content(sprite_rgba, padding)

            # Save sprite
            output_path = row_dir / f"sprite_{col_num:03d}.png"
            cv2.imwrite(str(output_path), sprite_rgba)
            total_sprites += 1

    print(f"\n{'='*60}")
    print(f"COMPLETE!")
    print(f"{'='*60}")
    print(f"Processed {len(rows)} rows")
    print(f"Extracted {total_sprites} sprites")
    print(f"Output: {output_dir}")
    print(f"{'='*60}\n")


def main():
    parser = argparse.ArgumentParser(
        description="Process sprite sheets by detecting grid lines and removing colored backgrounds",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Basic usage with k-means (recommended)
  python sprite_sheet_processor.py pkmn_people.png

  # Use color frequency method instead
  python sprite_sheet_processor.py pkmn_people.png --method color_freq

  # Adjust k-means clusters for complex backgrounds
  python sprite_sheet_processor.py pkmn_people.png --clusters 4

  # Don't crop sprites to content
  python sprite_sheet_processor.py pkmn_people.png --no-crop
        """
    )

    parser.add_argument(
        "sprite_sheet",
        help="Path to the sprite sheet image"
    )

    parser.add_argument(
        "-o", "--output",
        help="Output directory (default: <sprite_sheet_name>_processed)"
    )

    parser.add_argument(
        "-m", "--method",
        choices=['kmeans', 'color_freq'],
        default='kmeans',
        help="Extraction method: 'kmeans' (best quality) or 'color_freq' (fastest) (default: kmeans)"
    )

    parser.add_argument(
        "-c", "--clusters",
        type=int,
        default=3,
        help="Number of color clusters for k-means method (2-5 typical, default: 3)"
    )

    parser.add_argument(
        "-t", "--tolerance",
        type=int,
        default=40,
        help="Color tolerance for color_freq method (20-60 typical, default: 40)"
    )

    parser.add_argument(
        "-w", "--white-threshold",
        type=int,
        default=240,
        help="Threshold for detecting white grid lines (200-255, default: 240)"
    )

    parser.add_argument(
        "--no-crop",
        action="store_true",
        help="Don't crop sprites to content (keeps full cell size)"
    )

    parser.add_argument(
        "-p", "--padding",
        type=int,
        default=1,
        help="Pixels of padding when cropping (default: 1)"
    )

    args = parser.parse_args()

    # Process the sprite sheet
    process_sprite_sheet(
        sheet_path=args.sprite_sheet,
        output_dir=args.output,
        method=args.method,
        n_clusters=args.clusters,
        tolerance=args.tolerance,
        white_threshold=args.white_threshold,
        crop=not args.no_crop,
        padding=args.padding
    )


if __name__ == "__main__":
    main()
