#!/usr/bin/env python3
"""Generate CBOR macropad config files for FreeCAD, CS2, Bitwig, Darktable, GIMP."""

import struct
import os

ICON_SIZE = 21

# ---------------------------------------------------------------------------
# 3×5 pixel bitmap font  (5 rows × 3-bit masks, bit2=left col)
# ---------------------------------------------------------------------------
FONT = {
    'A': [0b010, 0b101, 0b111, 0b101, 0b101],
    'B': [0b110, 0b101, 0b110, 0b101, 0b110],
    'C': [0b011, 0b100, 0b100, 0b100, 0b011],
    'D': [0b110, 0b101, 0b101, 0b101, 0b110],
    'E': [0b111, 0b100, 0b110, 0b100, 0b111],
    'F': [0b111, 0b100, 0b110, 0b100, 0b100],
    'G': [0b011, 0b100, 0b101, 0b101, 0b011],
    'H': [0b101, 0b101, 0b111, 0b101, 0b101],
    'I': [0b111, 0b010, 0b010, 0b010, 0b111],
    'J': [0b011, 0b001, 0b001, 0b101, 0b010],
    'K': [0b101, 0b101, 0b110, 0b101, 0b101],
    'L': [0b100, 0b100, 0b100, 0b100, 0b111],
    'M': [0b101, 0b111, 0b101, 0b101, 0b101],
    'N': [0b101, 0b111, 0b101, 0b101, 0b101],
    'O': [0b010, 0b101, 0b101, 0b101, 0b010],
    'P': [0b110, 0b101, 0b110, 0b100, 0b100],
    'Q': [0b010, 0b101, 0b101, 0b111, 0b011],
    'R': [0b110, 0b101, 0b110, 0b101, 0b101],
    'S': [0b011, 0b100, 0b010, 0b001, 0b110],
    'T': [0b111, 0b010, 0b010, 0b010, 0b010],
    'U': [0b101, 0b101, 0b101, 0b101, 0b011],
    'V': [0b101, 0b101, 0b101, 0b010, 0b010],
    'W': [0b101, 0b101, 0b101, 0b111, 0b101],
    'X': [0b101, 0b101, 0b010, 0b101, 0b101],
    'Y': [0b101, 0b101, 0b010, 0b010, 0b010],
    'Z': [0b111, 0b001, 0b010, 0b100, 0b111],
    '0': [0b010, 0b101, 0b101, 0b101, 0b010],
    '1': [0b010, 0b110, 0b010, 0b010, 0b111],
    '2': [0b110, 0b001, 0b010, 0b100, 0b111],
    '3': [0b110, 0b001, 0b010, 0b001, 0b110],
    '4': [0b101, 0b101, 0b111, 0b001, 0b001],
    '5': [0b111, 0b100, 0b110, 0b001, 0b110],
    '6': [0b011, 0b100, 0b110, 0b101, 0b010],
    '7': [0b111, 0b001, 0b010, 0b010, 0b010],
    '8': [0b010, 0b101, 0b010, 0b101, 0b010],
    '9': [0b010, 0b101, 0b011, 0b001, 0b110],
    '+': [0b000, 0b010, 0b111, 0b010, 0b000],
    '-': [0b000, 0b000, 0b111, 0b000, 0b000],
    '>': [0b100, 0b010, 0b001, 0b010, 0b100],
    '<': [0b001, 0b010, 0b100, 0b010, 0b001],
    '!': [0b010, 0b010, 0b010, 0b000, 0b010],
    '?': [0b110, 0b001, 0b010, 0b000, 0b010],
    ' ': [0b000, 0b000, 0b000, 0b000, 0b000],
    '.': [0b000, 0b000, 0b000, 0b000, 0b010],
    '#': [0b101, 0b111, 0b101, 0b111, 0b101],
    '*': [0b000, 0b101, 0b010, 0b101, 0b000],
}


def make_text_icon(text, bg=0, fg=1, W=ICON_SIZE, H=ICON_SIZE):
    """Render up to 3 chars of text to a W×H flat pixel list (0/1)."""
    chars = [c for c in text[:3].upper() if c in FONT]
    grid = [[bg] * W for _ in range(H)]
    if not chars:
        return [grid[y][x] for y in range(H) for x in range(W)]
    n = len(chars)
    scale = 2 if n <= 2 else 1
    cw, ch, sp = 3 * scale, 5 * scale, scale
    total_w = n * cw + (n - 1) * sp
    sx = max(0, (W - total_w) // 2)
    sy = max(0, (H - ch) // 2)
    for ci, ch_char in enumerate(chars):
        rows = FONT[ch_char]
        ox = sx + ci * (cw + sp)
        for ri, mask in enumerate(rows):
            for bi in range(3):
                bit = (mask >> (2 - bi)) & 1
                val = fg if bit else bg
                for dy in range(scale):
                    for dx in range(scale):
                        y, x = sy + ri * scale + dy, ox + bi * scale + dx
                        if 0 <= y < H and 0 <= x < W:
                            grid[y][x] = val
    return [grid[y][x] for y in range(H) for x in range(W)]


def make_bmp(pixels, W=ICON_SIZE, H=ICON_SIZE):
    """Encode a flat 0/1 pixel array as a 1-bit BMP (index0=black, index1=white)."""
    HDR, INFO, PAL = 14, 40, 8
    rb = (W + 7) // 8
    prb = ((rb + 3) // 4) * 4
    pds = prb * H
    fsz = HDR + INFO + PAL + pds
    doff = HDR + INFO + PAL
    buf = bytearray(fsz)
    buf[0], buf[1] = 0x42, 0x4d
    struct.pack_into('<I', buf, 2, fsz)
    struct.pack_into('<I', buf, 10, doff)
    struct.pack_into('<I', buf, 14, INFO)
    struct.pack_into('<i', buf, 18, W)
    struct.pack_into('<i', buf, 22, H)
    struct.pack_into('<H', buf, 26, 1)
    struct.pack_into('<H', buf, 28, 1)
    struct.pack_into('<I', buf, 34, pds)
    struct.pack_into('<I', buf, 38, 0x0b13)
    struct.pack_into('<I', buf, 42, 0x0b13)
    struct.pack_into('<I', buf, 46, 2)
    # color table: index0=black, index1=white (BGRA)
    buf[54:62] = bytes([0, 0, 0, 0, 255, 255, 255, 0])
    for row in range(H):
        brow = H - 1 - row
        rs = doff + brow * prb
        for col in range(W):
            if pixels[row * W + col]:
                buf[rs + (col >> 3)] |= 1 << (7 - (col & 7))
    return bytes(buf)


# ---------------------------------------------------------------------------
# CBOR helpers
# ---------------------------------------------------------------------------

def cu(n):
    if n <= 0x17: return bytes([n])
    if n <= 0xff: return bytes([0x18, n])
    if n <= 0xffff: return bytes([0x19, n >> 8, n & 0xff])
    return bytes([0x1a]) + n.to_bytes(4, 'big')

def ct(s):
    b = s.encode()
    n = len(b)
    h = bytes([0x60 | n]) if n <= 0x17 else bytes([0x78, n]) if n <= 0xff else bytes([0x79]) + n.to_bytes(2, 'big')
    return h + b

def ca(items):
    n = len(items)
    h = bytes([0x80 | n]) if n <= 0x17 else bytes([0x98, n]) if n <= 0xff else bytes([0x99]) + n.to_bytes(2, 'big')
    return h + b''.join(items)

def cm(pairs):
    n = len(pairs)
    h = bytes([0xa0 | n]) if n <= 0x17 else bytes([0xb8, n])
    return h + b''.join(ct(k) + v for k, v in pairs)

NULL = bytes([0xf6])


def enc_icon(bmp_bytes):
    """Encode BMP as CBOR array of uint8 values (matches ciborium's Vec<u8> encoding)."""
    return ca([cu(b) for b in bmp_bytes])


def enc_ks(strokes):
    """Encode Vec<Vec<KeyboardCode>> → CBOR array of arrays of text strings."""
    return ca([ca([ct(k) for k in chord]) for chord in strokes])


def enc_led(r, g, b):
    return cm([('r', cu(r)), ('g', cu(g)), ('b', cu(b))])


# row=1 → white bg / black text (buttons 0-4)
# row=0 or 2 → black bg / white text (encoders, buttons 5-9)
def _icon(text, row):
    bg, fg = (1, 0) if row == 1 else (0, 1)
    return enc_icon(make_bmp(make_text_icon(text, bg=bg, fg=fg)))

def btn(text, ks, row=1):
    return cm([('display_text', ct(text)), ('display_icon', _icon(text, row)), ('keystroke', enc_ks(ks))])

def enc(text, left, right, push, row=0):
    return cm([('display_text', ct(text)), ('display_icon', _icon(text, row)),
               ('keystroke_left', enc_ks(left)), ('keystroke_right', enc_ks(right)),
               ('keystroke_push', enc_ks(push))])

def menc(text, left, right, row=0):
    return cm([('display_text', ct(text)), ('display_icon', _icon(text, row)),
               ('keystroke_left', enc_ks(left)), ('keystroke_right', enc_ks(right))])

def config(name, buttons, menu_e, encoders, leds):
    return cm([
        ('name', ct(name)),
        ('buttons', ca(buttons)),
        ('menu_encoder', menu_e),
        ('encoders', ca(encoders)),
        ('leds', ca([enc_led(r, g, b) for r, g, b in leds])),
    ])

NOP = [['NoEventIndicated']]

# ---------------------------------------------------------------------------
# FreeCAD
# ---------------------------------------------------------------------------
def freecad():
    btns = [
        btn('Fit',  [['V'], ['F']],                                     row=1),  # Fit All
        btn('FitS', [['V'], ['S']],                                     row=1),  # Fit Selected
        btn('Nrm',  [['V'], ['Keyboard1']],                             row=1),  # Normal draw style
        btn('Wire', [['V'], ['Keyboard5']],                             row=1),  # Wireframe draw style
        btn('Home', [['Keypad0']],                                      row=1),  # Isometric home view
        btn('Undo', [['LeftControl', 'Z']],                             row=2),
        btn('Redo', [['LeftControl', 'Y']],                             row=2),
        btn('Vis',  [['Space']],                                        row=2),  # Toggle visibility
        btn('Imp',  [['LeftControl', 'I']],                             row=2),  # Import
        btn('Exp',  [['LeftControl', 'E']],                             row=2),  # Export
    ]
    e1   = enc('Zoom', [['KeypadSubtract']], [['KeypadAdd']], NOP)
    e2   = enc('UndR', [['LeftControl', 'Z']], [['LeftControl', 'Y']], NOP)
    me   = menc('Vol',  [['VolumeDown']], [['VolumeUp']])
    leds = [(203,51,59)]*4 + [(65,143,222)]*4   # FreeCAD red + blue
    return config('FreeCAD', btns, me, [e1, e2], leds)

# ---------------------------------------------------------------------------
# Counter-Strike 2
# ---------------------------------------------------------------------------
def cs2():
    btns = [
        btn('Buy',  [['B']],                                            row=1),  # Open buy menu
        btn('AK',   [['B'], ['Keyboard4'], ['Keyboard2']],              row=1),  # Buy AK-47 / M4
        btn('AWP',  [['B'], ['Keyboard4'], ['Keyboard1']],              row=1),  # Buy AWP
        btn('Vest', [['B'], ['Keyboard5'], ['Keyboard2']],              row=1),  # Armor+helmet
        btn('HE',   [['B'], ['Keyboard6'], ['Keyboard1']],              row=1),  # HE grenade
        btn('Smk',  [['B'], ['Keyboard6'], ['Keyboard3']],              row=2),  # Smoke grenade
        btn('Fls',  [['B'], ['Keyboard6'], ['Keyboard2']],              row=2),  # Flashbang
        btn('MIC',  [['K']],                                            row=2),  # Push-to-talk
        btn('Tab',  [['Tab']],                                          row=2),  # Scoreboard
        btn('CNS',  [['Grave']],                                        row=2),  # Console
    ]
    e1   = enc('Wpn',  [['Keyboard1']], [['Keyboard2']], [['Keyboard3']])   # slot 1/2/knife
    e2   = enc('Vol',  [['VolumeDown']], [['VolumeUp']], [['Mute']])
    me   = menc('Vol', [['VolumeDown']], [['VolumeUp']])
    leds = [(252,172,25)]*8   # CS2 yellow
    return config('Counter-Strike 2', btns, me, [e1, e2], leds)

# ---------------------------------------------------------------------------
# Bitwig
# ---------------------------------------------------------------------------
def bitwig():
    btns = [
        btn('Play', [['Space']],                                        row=1),
        btn('Rec',  [['LeftShift', 'Space']],                           row=1),  # Activate recording
        btn('Loop', [['L']],                                            row=1),  # Toggle loop
        btn('Undo', [['LeftControl', 'Z']],                             row=1),
        btn('Redo', [['LeftControl', 'Y']],                             row=1),
        btn('Qnt',  [['U']],                                            row=2),  # Quantize
        btn('Dup',  [['LeftControl', 'D']],                             row=2),  # Duplicate
        btn('Save', [['LeftControl', 'S']],                             row=2),
        btn('Fit',  [['LeftShift', 'F']],                               row=2),  # Zoom to fit
        btn('Trk',  [['LeftControl', 'T']],                             row=2),  # New track
    ]
    e1   = enc('Zoom', [['LeftControl', 'Minus']], [['LeftControl', 'Equal']], [['LeftShift', 'F']])
    e2   = enc('Scrl', [['LeftArrow']], [['RightArrow']], [['LeftControl', 'S']])
    me   = menc('Vol', [['VolumeDown']], [['VolumeUp']])
    leds = [(232,65,24)]*8   # Bitwig orange-red
    return config('Bitwig', btns, me, [e1, e2], leds)

# ---------------------------------------------------------------------------
# Darktable
# ---------------------------------------------------------------------------
def darktable():
    btns = [
        btn('Drm',  [['D']],                                            row=1),  # Switch to darkroom
        btn('Ltb',  [['L']],                                            row=1),  # Switch to lighttable
        btn('Exp',  [['LeftControl', 'E']],                             row=1),  # Export
        btn('Undo', [['LeftControl', 'Z']],                             row=1),
        btn('Redo', [['LeftControl', 'Y']],                             row=1),
        btn('Fit',  [['F']],                                            row=2),  # Zoom fit
        btn('1',    [['Keyboard1']],                                    row=2),  # 1 star
        btn('3',    [['Keyboard3']],                                    row=2),  # 3 stars
        btn('5',    [['Keyboard5']],                                    row=2),  # 5 stars
        btn('Rej',  [['R']],                                            row=2),  # Reject image
    ]
    e1   = enc('Zoom', [['Minus']], [['Equal']], [['F']])
    e2   = enc('Nav',  [['LeftArrow']], [['RightArrow']], [['ReturnEnter']])
    me   = menc('Vol', [['VolumeDown']], [['VolumeUp']])
    leds = [(180,30,30)]*4 + [(70,70,70)]*4   # dark red + grey
    return config('Darktable', btns, me, [e1, e2], leds)

# ---------------------------------------------------------------------------
# GIMP
# ---------------------------------------------------------------------------
def gimp():
    btns = [
        btn('Undo', [['LeftControl', 'Z']],                             row=1),
        btn('Redo', [['LeftControl', 'Y']],                             row=1),
        btn('SelA', [['LeftControl', 'A']],                             row=1),  # Select all
        btn('SelN', [['LeftShift', 'LeftControl', 'A']],                row=1),  # Select none
        btn('ExpA', [['LeftShift', 'LeftControl', 'E']],                row=1),  # Export As
        btn('LyrN', [['LeftShift', 'LeftControl', 'N']],                row=2),  # New layer
        btn('Copy', [['LeftControl', 'C']],                             row=2),
        btn('Pst',  [['LeftControl', 'V']],                             row=2),  # Paste
        btn('Crop', [['C']],                                            row=2),  # Crop tool
        btn('Fit',  [['LeftShift', 'LeftControl', 'J']],                row=2),  # Zoom to fit
    ]
    e1   = enc('Zoom', [['Minus']], [['Equal']], [['LeftShift', 'LeftControl', 'J']])
    e2   = enc('Lyr',  [['PageUp']], [['PageDown']], [['LeftShift', 'LeftControl', 'N']])
    me   = menc('Vol', [['VolumeDown']], [['VolumeUp']])
    leds = [(74,101,150)]*4 + [(45,80,140)]*4   # GIMP blue tones
    return config('GIMP', btns, me, [e1, e2], leds)


# ---------------------------------------------------------------------------
# Custom icon pixel generators  (0 = background, 1 = foreground shape)
# ---------------------------------------------------------------------------

def make_play_pixels(W=ICON_SIZE, H=ICON_SIZE):
    """Right-pointing solid triangle (play button)."""
    grid = [[0] * W for _ in range(H)]
    cy = H // 2
    x_left = 4
    half_h = cy - 2
    x_span = W - x_left - 5
    for row in range(H):
        dy = abs(row - cy)
        if dy <= half_h:
            w = round(x_span * (1 - dy / half_h))
            for x in range(x_left, x_left + w + 1):
                if 0 <= x < W:
                    grid[row][x] = 1
    return [grid[y][x] for y in range(H) for x in range(W)]


def make_record_pixels(W=ICON_SIZE, H=ICON_SIZE):
    """Filled circle (record button)."""
    grid = [[0] * W for _ in range(H)]
    cx, cy = W // 2, H // 2
    r = 7
    for y in range(H):
        for x in range(W):
            if (x - cx) ** 2 + (y - cy) ** 2 <= r * r:
                grid[y][x] = 1
    return [grid[y][x] for y in range(H) for x in range(W)]


def make_tile_left_pixels(W=ICON_SIZE, H=ICON_SIZE):
    """Split-screen outline with left half filled (tile left)."""
    grid = [[0] * W for _ in range(H)]
    mid = W // 2
    for x in range(2, W - 2):
        grid[2][x] = 1
        grid[H - 3][x] = 1
    for y in range(2, H - 2):
        grid[y][2] = 1
        grid[y][W - 3] = 1
        grid[y][mid] = 1
    for y in range(3, H - 3):
        for x in range(3, mid):
            grid[y][x] = 1
    return [grid[y][x] for y in range(H) for x in range(W)]


def make_tile_right_pixels(W=ICON_SIZE, H=ICON_SIZE):
    """Split-screen outline with right half filled (tile right)."""
    grid = [[0] * W for _ in range(H)]
    mid = W // 2
    for x in range(2, W - 2):
        grid[2][x] = 1
        grid[H - 3][x] = 1
    for y in range(2, H - 2):
        grid[y][2] = 1
        grid[y][W - 3] = 1
        grid[y][mid] = 1
    for y in range(3, H - 3):
        for x in range(mid + 1, W - 3):
            grid[y][x] = 1
    return [grid[y][x] for y in range(H) for x in range(W)]


def _custom_icon(pixels, row):
    """Apply row color scheme (row=1 → white bg/black fg, else black bg/white fg)."""
    bg, fg = (1, 0) if row == 1 else (0, 1)
    return enc_icon(make_bmp([fg if p else bg for p in pixels]))


def btn_px(text, pixels, ks, row=1):
    """Button with a custom pixel art icon."""
    return cm([('display_text', ct(text)), ('display_icon', _custom_icon(pixels, row)), ('keystroke', enc_ks(ks))])


# ---------------------------------------------------------------------------
# FL Studio
# ---------------------------------------------------------------------------
def fl_studio():
    play_px = make_play_pixels()
    rec_px  = make_record_pixels()
    btns = [
        btn_px('Play', play_px, [['Space']],                            row=1),  # Play / Pause
        btn_px('Rec',  rec_px,  [['R']],                                row=1),  # Arm recording
        btn('Loop', [['L']],                                            row=1),  # Toggle loop
        btn('Undo', [['LeftControl', 'Z']],                             row=1),
        btn('Redo', [['LeftControl', 'Y']],                             row=1),
        btn('Pno',  [['F7']],                                           row=2),  # Piano Roll
        btn('Mxr',  [['F9']],                                           row=2),  # Mixer
        btn('Chr',  [['F6']],                                           row=2),  # Channel Rack
        btn('Ply',  [['F5']],                                           row=2),  # Playlist
        btn('Save', [['LeftControl', 'S']],                             row=2),
    ]
    e1   = enc('Zoom', [['LeftControl', 'Minus']], [['LeftControl', 'Equal']], NOP)
    e2   = enc('Nav',  [['LeftArrow']], [['RightArrow']], [['Space']])  # Timeline nav, push=play
    me   = menc('Vol', [['VolumeDown']], [['VolumeUp']])
    leds = [(255, 145, 0)] * 8   # FL Studio amber-orange
    return config('FL Studio', btns, me, [e1, e2], leds)


# ---------------------------------------------------------------------------
# COSMIC Desktop
# ---------------------------------------------------------------------------
def cosmic():
    tile_l = make_tile_left_pixels()
    tile_r = make_tile_right_pixels()
    btns = [
        btn_px('TilL', tile_l, [['LeftGUI', 'LeftArrow']],             row=1),  # Tile window left
        btn_px('TilR', tile_r, [['LeftGUI', 'RightArrow']],            row=1),  # Tile window right
        btn('Max',  [['LeftGUI', 'UpArrow']],                           row=1),  # Maximize
        btn('Flt',  [['LeftGUI', 'DownArrow']],                         row=1),  # Float / restore
        btn('Ovw',  [['LeftGUI', 'W']],                                 row=1),  # Workspace overview
        btn('WS<',  [['LeftGUI', 'LeftControl', 'LeftArrow']],          row=2),  # Prev workspace
        btn('WS>',  [['LeftGUI', 'LeftControl', 'RightArrow']],         row=2),  # Next workspace
        btn('MvL',  [['LeftGUI', 'LeftShift', 'LeftArrow']],            row=2),  # Move window to prev ws
        btn('MvR',  [['LeftGUI', 'LeftShift', 'RightArrow']],           row=2),  # Move window to next ws
        btn('Term', [['LeftControl', 'LeftAlt', 'T']],                  row=2),  # Open terminal
    ]
    e1   = enc('WS',   [['LeftGUI', 'LeftControl', 'LeftArrow']], [['LeftGUI', 'LeftControl', 'RightArrow']], NOP)
    e2   = enc('Vol',  [['VolumeDown']], [['VolumeUp']], [['Mute']])
    me   = menc('Vol', [['VolumeDown']], [['VolumeUp']])
    leds = [(0, 185, 215)] * 4 + [(0, 120, 165)] * 4   # COSMIC teal / deep teal
    return config('COSMIC', btns, me, [e1, e2], leds)


# ---------------------------------------------------------------------------
# Firefox
# ---------------------------------------------------------------------------
def firefox():
    btns = [
        btn('NewT', [['LeftControl', 'T']],                             row=1),  # New tab
        btn('ClsT', [['LeftControl', 'W']],                             row=1),  # Close tab
        btn('ROpn', [['LeftControl', 'LeftShift', 'T']],                row=1),  # Reopen closed tab
        btn('Back', [['LeftAlt', 'LeftArrow']],                         row=1),  # Navigate back
        btn('Fwd',  [['LeftAlt', 'RightArrow']],                        row=1),  # Navigate forward
        btn('Reld', [['LeftControl', 'R']],                             row=2),  # Reload
        btn('Find', [['LeftControl', 'F']],                             row=2),  # Find in page
        btn('Addr', [['LeftControl', 'L']],                             row=2),  # Focus address bar
        btn('Bkmk', [['LeftControl', 'D']],                             row=2),  # Bookmark page
        btn('Priv', [['LeftControl', 'LeftShift', 'P']],                row=2),  # New private window
    ]
    e1   = enc('Tab',  [['LeftControl', 'LeftShift', 'Tab']], [['LeftControl', 'Tab']], NOP)  # Prev/next tab
    e2   = enc('Zoom', [['LeftControl', 'Minus']], [['LeftControl', 'Equal']], [['LeftControl', 'Keyboard0']])
    me   = menc('Vol', [['VolumeDown']], [['VolumeUp']])
    leds = [(235, 50, 10)] * 8   # Firefox red-orange
    return config('Firefox', btns, me, [e1, e2], leds)


# ---------------------------------------------------------------------------
# Write files
# ---------------------------------------------------------------------------
if __name__ == '__main__':
    out = os.path.join(os.path.dirname(__file__))
    files = [
        ('FREECAD.cfg',  freecad()),
        ('CS2.cfg',      cs2()),
        ('BITWIG.cfg',   bitwig()),
        ('DARKTBL.cfg',   darktable()),
        ('GIMP.cfg',     gimp()),
        ('FLSTUDIO.cfg',   fl_studio()),
        ('COSMIC.cfg',   cosmic()),
        ('FIREFOX.cfg',  firefox()),
    ]
    for fname, data in files:
        path = os.path.join(out, fname)
        with open(path, 'wb') as f:
            f.write(data)
        print(f'{fname:16s}  {len(data):5d} bytes')
