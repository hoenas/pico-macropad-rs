import init, {
  default_config,
  export_config_to_cbor,
  import_config_from_cbor,
} from '../pkg/pico_macropad_wasm.js';

const editorGrid = document.getElementById('editorGrid');
const importButton = document.getElementById('importButton');
const exportButton = document.getElementById('exportButton');
const newConfigButton = document.getElementById('newConfigButton');
const configFileInput = document.getElementById('configFileInput');
const iconEditorModal = document.getElementById('iconEditorModal');
const closeIconEditorBtn = document.getElementById('closeIconEditor');
const cancelIconButton = document.getElementById('cancelIconButton');
const saveIconButton = document.getElementById('saveIconButton');
const editorCanvas = document.getElementById('editorCanvas');
const previewCanvas = document.getElementById('previewCanvas');
const toolSelect = document.getElementById('toolSelect');
const colorSelect = document.getElementById('colorSelect');
const brushSizeSelect = document.getElementById('brushSizeSelect');
const textInput = document.getElementById('textInput');
const fillCanvasButton = document.getElementById('fillCanvas');
const clearCanvasButton = document.getElementById('clearCanvas');
const iconFileInput = document.getElementById('iconFileInput');

const editorCtx = editorCanvas.getContext('2d');
const previewCtx = previewCanvas.getContext('2d');

let config = null;
let currentIconTarget = null;
let currentIconCanvas = null; // card canvas to refresh on save
let iconPixels = new Uint8Array(22 * 22);
let iconColor = 1;
let isMouseDown = false;

// ---------------------------------------------------------------------------
// Config helpers
// ---------------------------------------------------------------------------

function createEmptyElement() {
  return {
    display_text: '',
    display_icon: null,
    keystroke: [],
    keystroke_left: [],
    keystroke_right: [],
    keystroke_push: [],
  };
}

function newConfig() {
  return {
    name: 'New configuration',
    buttons: Array.from({ length: 10 }, createEmptyElement),
    encoders: Array.from({ length: 2 }, createEmptyElement),
    menu_encoder: createEmptyElement(),
    leds: Array.from({ length: 8 }, () => ({ r: 0, g: 0, b: 0 })),
  };
}

// ---------------------------------------------------------------------------
// Keystroke serialisation
// Each chord is one line; keys within a chord are comma-separated KeyboardCode
// variant name strings, e.g. "LeftControl,C" or "ReturnEnter".
// ---------------------------------------------------------------------------

function serializeKeystrokes(strokes) {
  return (strokes || []).map(chord => chord.join(',')).join('\n');
}

function parseKeystrokeLines(content) {
  return content
    .split(/\r?\n/)
    .map(line => line.trim())
    .filter(line => line.length > 0)
    .map(line => line.split(/\s*,\s*/).map(token => token.trim()).filter(Boolean));
}

// ---------------------------------------------------------------------------
// BMP encode / decode (1-bit, 22×22)
// ---------------------------------------------------------------------------

function encodeBmp(pixels) {
  const headerSize = 14;
  const infoSize = 40;
  const paletteSize = 8; // 2 colours × 4 bytes
  const rowBytes = Math.ceil(22 / 8);
  const paddedRowBytes = ((rowBytes + 3) >> 2) << 2;
  const pixelDataSize = paddedRowBytes * 22;
  const fileSize = headerSize + infoSize + paletteSize + pixelDataSize;
  const bytes = new Uint8Array(fileSize);

  // BMP file header
  bytes[0] = 0x42; bytes[1] = 0x4d; // 'BM'
  bytes[2] = fileSize & 0xff;
  bytes[3] = (fileSize >> 8) & 0xff;
  bytes[4] = (fileSize >> 16) & 0xff;
  bytes[5] = (fileSize >> 24) & 0xff;
  const dataOffset = headerSize + infoSize + paletteSize;
  bytes[10] = dataOffset & 0xff;
  bytes[11] = (dataOffset >> 8) & 0xff;

  // BITMAPINFOHEADER
  bytes[14] = infoSize;
  bytes[18] = 22; bytes[22] = 22; // width, height (bottom-up)
  bytes[26] = 1;  // planes
  bytes[28] = 1;  // bits per pixel
  bytes[34] = pixelDataSize & 0xff;
  bytes[35] = (pixelDataSize >> 8) & 0xff;
  bytes[38] = 0x13; bytes[39] = 0x0b; // ~72 dpi x
  bytes[42] = 0x13; bytes[43] = 0x0b; // ~72 dpi y
  bytes[46] = 2; // colours used

  // Colour table: index 0 = black, index 1 = white
  bytes[54] = 0;   bytes[55] = 0;   bytes[56] = 0;   bytes[57] = 0;
  bytes[58] = 255; bytes[59] = 255; bytes[60] = 255; bytes[61] = 0;

  // Pixel data (bottom-up row order)
  for (let row = 0; row < 22; row++) {
    const rowStart = dataOffset + (21 - row) * paddedRowBytes;
    for (let col = 0; col < 22; col++) {
      const bit = pixels[row * 22 + col] ? 1 : 0;
      const bytePos = rowStart + (col >> 3);
      const bitPos = 7 - (col & 7);
      bytes[bytePos] |= bit << bitPos;
    }
  }

  return bytes;
}

function decodeBmp(bytes) {
  if (!(bytes instanceof Uint8Array)) {
    bytes = new Uint8Array(bytes);
  }
  if (bytes[0] !== 0x42 || bytes[1] !== 0x4d) return null;
  const width = bytes[18] | (bytes[19] << 8) | (bytes[20] << 16) | (bytes[21] << 24);
  const height = bytes[22] | (bytes[23] << 8) | (bytes[24] << 16) | (bytes[25] << 24);
  const bitsPerPixel = bytes[28] | (bytes[29] << 8);
  const dataOffset = bytes[10] | (bytes[11] << 8) | (bytes[12] << 16) | (bytes[13] << 24);
  if (width !== 22 || Math.abs(height) !== 22) return null;

  const pixelArray = new Uint8Array(22 * 22);
  const rowBytes = Math.ceil(width / 8);
  const paddedRowBytes = ((rowBytes + 3) >> 2) << 2;
  const bottomUp = height > 0;

  if (bitsPerPixel === 1) {
    for (let row = 0; row < 22; row++) {
      const srcRow = bottomUp ? 21 - row : row;
      const rowStart = dataOffset + srcRow * paddedRowBytes;
      for (let col = 0; col < 22; col++) {
        const byteIndex = rowStart + (col >> 3);
        const bitIndex = 7 - (col & 7);
        pixelArray[row * 22 + col] = (bytes[byteIndex] >> bitIndex) & 1;
      }
    }
  } else if (bitsPerPixel === 24) {
    const paddedRow24 = ((width * 3 + 3) >> 2) << 2;
    for (let row = 0; row < 22; row++) {
      const srcRow = bottomUp ? 21 - row : row;
      const rowStart = dataOffset + srcRow * paddedRow24;
      for (let col = 0; col < 22; col++) {
        const px = rowStart + col * 3;
        const luma = (bytes[px] + bytes[px + 1] + bytes[px + 2]) / 3;
        pixelArray[row * 22 + col] = luma > 127 ? 1 : 0;
      }
    }
  } else {
    return null;
  }

  return pixelArray;
}

// ---------------------------------------------------------------------------
// Icon canvas rendering
// ---------------------------------------------------------------------------

function renderIconCanvas(canvas, iconBytes) {
  const ctx = canvas.getContext('2d');
  ctx.fillStyle = '#000';
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  if (!iconBytes) return;

  const blob = new Blob([new Uint8Array(iconBytes)], { type: 'image/bmp' });
  const url = URL.createObjectURL(blob);
  const image = new Image();
  image.onload = () => {
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(image, 0, 0, canvas.width, canvas.height);
    URL.revokeObjectURL(url);
  };
  image.src = url;
}

// Render a pixel array (defaults to iconPixels) to the editor and preview canvases.
function updateEditorCanvas(pixels = iconPixels) {
  const imageData = editorCtx.createImageData(22, 22);
  for (let i = 0; i < pixels.length; i++) {
    const v = pixels[i] ? 255 : 0;
    imageData.data[i * 4 + 0] = v;
    imageData.data[i * 4 + 1] = v;
    imageData.data[i * 4 + 2] = v;
    imageData.data[i * 4 + 3] = 255;
  }
  const offscreen = document.createElement('canvas');
  offscreen.width = 22; offscreen.height = 22;
  offscreen.getContext('2d').putImageData(imageData, 0, 0);

  editorCtx.imageSmoothingEnabled = false;
  editorCtx.drawImage(offscreen, 0, 0, 220, 220);

  // Only sync the small preview canvas when showing committed pixels.
  if (pixels === iconPixels) {
    const pData = previewCtx.createImageData(22, 22);
    pData.data.set(imageData.data);
    previewCtx.putImageData(pData, 0, 0);
  }
}

// ---------------------------------------------------------------------------
// Editor grid
// ---------------------------------------------------------------------------

function createKeystrokeSection(element) {
  const container = document.createElement('div');

  function makeArea(label, key) {
    const group = document.createElement('div');
    group.className = 'field-group';
    group.innerHTML = `<label>${label}</label>
      <textarea placeholder="One chord per line, key names e.g. LeftControl,C">${serializeKeystrokes(element[key])}</textarea>`;
    group.querySelector('textarea').addEventListener('input', (e) => {
      element[key] = parseKeystrokeLines(e.target.value);
    });
    return group;
  }

  if ('keystroke_left' in element && 'keystroke_right' in element) {
    container.appendChild(makeArea('Left', 'keystroke_left'));
    container.appendChild(makeArea('Right', 'keystroke_right'));
    if ('keystroke_push' in element) {
      container.appendChild(makeArea('Push', 'keystroke_push'));
    }
  } else {
    container.appendChild(makeArea('Keystroke', 'keystroke'));
  }

  return container;
}

function createCard(title, element, { titleOnly = false } = {}) {
  const card = document.createElement('div');
  card.className = 'card';

  const heading = document.createElement('h3');
  heading.textContent = title;
  card.appendChild(heading);

  // Display text field
  const titleGroup = document.createElement('div');
  titleGroup.className = 'field-group';
  titleGroup.innerHTML = `<label>Title</label><input type="text" value="${(element.display_text || '').replace(/"/g, '&quot;')}" />`;
  const titleInput = titleGroup.querySelector('input');
  titleInput.addEventListener('input', () => { element.display_text = titleInput.value; });
  card.appendChild(titleGroup);

  if (!titleOnly) {
    card.appendChild(createKeystrokeSection(element));

    // Icon preview + editor button
    const iconGroup = document.createElement('div');
    iconGroup.className = 'field-group icon-preview';
    iconGroup.innerHTML = `<label>Icon</label>
      <div class="preview-box"><canvas width="22" height="22"></canvas></div>
      <button type="button">Edit icon</button>`;
    const iconCanvas = iconGroup.querySelector('canvas');
    renderIconCanvas(iconCanvas, element.display_icon);
    iconGroup.querySelector('button').addEventListener('click', () => openIconEditor(element, iconCanvas));
    card.appendChild(iconGroup);
  }

  return card;
}

function renderEditorGrid() {
  editorGrid.innerHTML = '';

  // Row 1: Encoder1, Encoder2, Title/Name, [empty], MenuEncoder
  const nameProxy = { display_text: config.name };
  const nameCard = createCard('Config name', nameProxy, { titleOnly: true });
  nameProxy.__onchange = () => { config.name = nameProxy.display_text; };
  nameCard.querySelector('input').addEventListener('input', () => { config.name = nameProxy.display_text; });

  editorGrid.appendChild(createCard('Encoder 1', config.encoders[0]));
  editorGrid.appendChild(createCard('Encoder 2', config.encoders[1]));
  editorGrid.appendChild(nameCard);

  const emptyCell = document.createElement('div');
  emptyCell.className = 'card empty';
  editorGrid.appendChild(emptyCell);

  editorGrid.appendChild(createCard('Menu encoder', config.menu_encoder));

  // Rows 2–3: Buttons 1–10
  config.buttons.forEach((btn, i) => {
    editorGrid.appendChild(createCard(`Button ${i + 1}`, btn));
  });
}

// ---------------------------------------------------------------------------
// Icon editor modal
// ---------------------------------------------------------------------------

function openIconEditor(element, cardCanvas) {
  currentIconTarget = element;
  currentIconCanvas = cardCanvas;

  const existing = element.display_icon
    ? decodeBmp(new Uint8Array(element.display_icon))
    : null;
  iconPixels = existing ?? new Uint8Array(22 * 22);
  iconColor = 1;
  colorSelect.value = 'white';
  toolSelect.value = 'brush';
  updateEditorControlState();
  updateEditorCanvas();
  iconEditorModal.classList.add('open');
}

function closeIconEditorModal() {
  iconEditorModal.classList.remove('open');
  currentIconTarget = null;
  currentIconCanvas = null;
}

function updateEditorControlState() {
  document.body.classList.toggle('tool-text', toolSelect.value === 'text');
}

function setPixel(x, y, value) {
  if (x < 0 || x >= 22 || y < 0 || y >= 22) return;
  iconPixels[y * 22 + x] = value;
}

function applyBrush(px, py) {
  const size = Number(brushSizeSelect.value);
  for (let dy = 0; dy < size; dy++) {
    for (let dx = 0; dx < size; dx++) {
      setPixel(px + dx, py + dy, iconColor);
    }
  }
}

// Font sizes for text tool: size 1 is smallest (≈5 px, fits 3–4 chars in 22 px).
const TEXT_FONT_SIZES = [5, 7, 10, 14];

function textFontSize() {
  return TEXT_FONT_SIZES[Math.min(Number(brushSizeSelect.value), 4) - 1];
}

// Render text into a copy of `basePixels` and return the new pixel array.
// Does not modify iconPixels.
function computeTextPixels(basePixels, text, px, py) {
  const result = new Uint8Array(basePixels);
  const tmp = document.createElement('canvas');
  tmp.width = 22; tmp.height = 22;
  const ctx = tmp.getContext('2d');
  // Start from a black background, paint existing pixels, then overlay text.
  ctx.fillStyle = '#000';
  ctx.fillRect(0, 0, 22, 22);
  for (let i = 0; i < 22 * 22; i++) {
    if (result[i]) {
      ctx.fillStyle = '#fff';
      ctx.fillRect(i % 22, Math.floor(i / 22), 1, 1);
    }
  }
  ctx.fillStyle = iconColor ? '#fff' : '#000';
  ctx.font = `${textFontSize()}px monospace`;
  ctx.textBaseline = 'top';
  ctx.fillText(text, px, py);
  const imgData = ctx.getImageData(0, 0, 22, 22);
  for (let row = 0; row < 22; row++) {
    for (let col = 0; col < 22; col++) {
      const idx = (row * 22 + col) * 4;
      if (imgData.data[idx] > 128) result[row * 22 + col] = iconColor;
    }
  }
  return result;
}

function applyText(text, px, py) {
  iconPixels = computeTextPixels(iconPixels, text, px, py);
}

function canvasCoords(clientX, clientY) {
  const rect = editorCanvas.getBoundingClientRect();
  return {
    x: Math.max(0, Math.min(21, Math.floor(((clientX - rect.left) / rect.width) * 22))),
    y: Math.max(0, Math.min(21, Math.floor(((clientY - rect.top) / rect.height) * 22))),
  };
}

function handlePointer(clientX, clientY) {
  const { x, y } = canvasCoords(clientX, clientY);
  if (toolSelect.value === 'text') {
    applyText(textInput.value || 'A', x, y);
  } else {
    applyBrush(x, y);
  }
  updateEditorCanvas();
}

editorCanvas.addEventListener('mousedown', (e) => { isMouseDown = true; handlePointer(e.clientX, e.clientY); });
document.addEventListener('mouseup', () => { isMouseDown = false; });
editorCanvas.addEventListener('mousemove', (e) => {
  if (isMouseDown) {
    handlePointer(e.clientX, e.clientY);
  } else if (toolSelect.value === 'text') {
    // Show a non-destructive live preview of the text at the cursor position.
    const { x, y } = canvasCoords(e.clientX, e.clientY);
    updateEditorCanvas(computeTextPixels(iconPixels, textInput.value || 'A', x, y));
  }
});
editorCanvas.addEventListener('mouseleave', () => {
  // Restore the canvas to the committed pixels when the cursor leaves.
  if (toolSelect.value === 'text') updateEditorCanvas();
});

toolSelect.addEventListener('change', updateEditorControlState);
colorSelect.addEventListener('change', () => { iconColor = colorSelect.value === 'white' ? 1 : 0; });

clearCanvasButton.addEventListener('click', () => { iconPixels.fill(0); updateEditorCanvas(); });
fillCanvasButton.addEventListener('click', () => { iconPixels.fill(1); updateEditorCanvas(); });

closeIconEditorBtn.addEventListener('click', closeIconEditorModal);
cancelIconButton.addEventListener('click', closeIconEditorModal);
iconEditorModal.addEventListener('click', (e) => { if (e.target === iconEditorModal) closeIconEditorModal(); });

saveIconButton.addEventListener('click', () => {
  if (!currentIconTarget) return;
  const bmp = encodeBmp(iconPixels);
  currentIconTarget.display_icon = Array.from(bmp);
  if (currentIconCanvas) renderIconCanvas(currentIconCanvas, bmp);
  closeIconEditorModal();
});

// Import icon from image file — convert to 22×22 B/W
iconFileInput.addEventListener('change', (e) => {
  const file = e.target.files[0];
  if (!file) return;
  const url = URL.createObjectURL(file);
  const img = new Image();
  img.onload = () => {
    const tmp = document.createElement('canvas');
    tmp.width = 22; tmp.height = 22;
    const ctx = tmp.getContext('2d');
    ctx.drawImage(img, 0, 0, 22, 22);
    const imgData = ctx.getImageData(0, 0, 22, 22);
    for (let i = 0; i < 22 * 22; i++) {
      const r = imgData.data[i * 4];
      const g = imgData.data[i * 4 + 1];
      const b = imgData.data[i * 4 + 2];
      iconPixels[i] = (r + g + b) / 3 > 127 ? 1 : 0;
    }
    updateEditorCanvas();
    URL.revokeObjectURL(url);
    e.target.value = '';
  };
  img.src = url;
});

// ---------------------------------------------------------------------------
// Import / Export
// ---------------------------------------------------------------------------

importButton.addEventListener('click', () => configFileInput.click());
configFileInput.addEventListener('change', async (e) => {
  const file = e.target.files[0];
  if (!file) return;
  const bytes = new Uint8Array(await file.arrayBuffer());
  config = await import_config_from_cbor(bytes);
  renderEditorGrid();
  e.target.value = '';
});

// Produce a FAT32 8.3-compliant base name from an arbitrary string.
function toFat32BaseName(name) {
  const sanitised = (name || '')
    .toUpperCase()
    .replace(/\s+/g, '_')
    .split('')
    .filter(c => /[A-Z0-9!#$%&'()\-@^_`{}~]/.test(c))
    .slice(0, 8)
    .join('');
  return sanitised || 'CONFIG';
}

exportButton.addEventListener('click', async () => {
  if (!config) return;
  const bytes = await export_config_to_cbor(config);
  const blob = new Blob([bytes], { type: 'application/octet-stream' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${toFat32BaseName(config.name)}.cfg`;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
});

newConfigButton.addEventListener('click', () => {
  config = newConfig();
  renderEditorGrid();
});

// ---------------------------------------------------------------------------
// Boot
// ---------------------------------------------------------------------------

async function initApp() {
  await init();
  config = await default_config();
  renderEditorGrid();
}

initApp().catch((err) => {
  console.error('Failed to initialise WASM module', err);
  editorGrid.textContent =
    'Failed to load the WASM module. Run: cd webapp/wasm && wasm-pack build --target web --out-dir ../pkg';
});
