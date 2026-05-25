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
const closeIconEditor = document.getElementById('closeIconEditor');
const saveIconButton = document.getElementById('saveIconButton');
const editorCanvas = document.getElementById('editorCanvas');
const previewCanvas = document.getElementById('previewCanvas');
const toolSelect = document.getElementById('toolSelect');
const colorSelect = document.getElementById('colorSelect');
const brushSizeSelect = document.getElementById('brushSizeSelect');
const textInput = document.getElementById('textInput');
const fillCanvasButton = document.getElementById('fillCanvas');
const clearCanvasButton = document.getElementById('clearCanvas');

const editorCtx = editorCanvas.getContext('2d');
const previewCtx = previewCanvas.getContext('2d');

let config = null;
let currentIconTarget = null;
let iconPixels = new Uint8Array(22 * 22);
let iconColor = 1;
let isMouseDown = false;

function clamp(value, min, max) {
  return Math.max(min, Math.min(max, value));
}

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
    buttons: Array.from({ length: 10 }, () => createEmptyElement()),
    encoders: Array.from({ length: 2 }, () => createEmptyElement()),
    menu_encoder: createEmptyElement(),
    leds: Array.from({ length: 8 }, () => ({ r: 0, g: 0, b: 0 })),
  };
}

function bytesToBase64(bytes) {
  let binary = '';
  const len = bytes.length;
  for (let i = 0; i < len; i += 1) {
    binary += String.fromCharCode(bytes[i]);
  }
  return window.btoa(binary);
}

function renderPreview(iconBytes) {
  previewCtx.clearRect(0, 0, 22, 22);
  previewCtx.fillStyle = '#000';
  previewCtx.fillRect(0, 0, 22, 22);

  if (!iconBytes) {
    return;
  }

  const blob = new Blob([iconBytes], { type: 'image/bmp' });
  const url = URL.createObjectURL(blob);
  const image = new Image();
  image.onload = () => {
    previewCtx.drawImage(image, 0, 0, 22, 22);
    URL.revokeObjectURL(url);
  };
  image.src = url;
}

function renderEditorGrid() {
  editorGrid.innerHTML = '';

  function createCard(title, element, path) {
    const card = document.createElement('div');
    card.className = 'card';

    const heading = document.createElement('h3');
    heading.textContent = title;
    card.appendChild(heading);

    const titleField = document.createElement('div');
    titleField.className = 'field-group';
    titleField.innerHTML = `
      <label>Title</label>
      <input type="text" value="${element.display_text || ''}" />
    `;
    const titleInput = titleField.querySelector('input');
    titleInput.addEventListener('input', () => {
      element.display_text = titleInput.value;
    });
    card.appendChild(titleField);

    const keystrokeField = createKeystrokeSection(element);
    card.appendChild(keystrokeField);

    const iconField = document.createElement('div');
    iconField.className = 'field-group icon-preview';
    iconField.innerHTML = `
      <label>Icon</label>
      <div class="preview-box"><canvas width="22" height="22"></canvas></div>
      <button type="button">Open icon editor</button>
    `;
    const iconCanvas = iconField.querySelector('canvas');
    const iconButton = iconField.querySelector('button');
    renderIconCanvas(iconCanvas, element.display_icon);
    iconButton.addEventListener('click', () => openIconEditor(element, iconCanvas));
    card.appendChild(iconField);

    return card;
  }

  const sections = [
    { title: 'Encoder 1', element: config.encoders[0], path: ['encoders', 0] },
    { title: 'Encoder 2', element: config.encoders[1], path: ['encoders', 1] },
    { title: 'Title section', element: { display_text: config.name, display_icon: null, keystroke: [] }, titleOnly: true },
    { title: 'Menu encoder', element: config.menu_encoder, path: ['menu_encoder'] },
    ...config.buttons.map((button, index) => ({ title: `Button ${index + 1}`, element: button, path: ['buttons', index] })),
  ];

  sections.forEach((section) => {
    const card = createCard(section.title, section.element, section.path);
    if (section.titleOnly) {
      const textInput = card.querySelector('input');
      textInput.addEventListener('input', () => {
        config.name = textInput.value;
      });
    }
    editorGrid.appendChild(card);
  });
}

function serializeKeystrokes(strokes) {
  const rows = strokes || [];
  return rows.map(line => line.join(',')).join('\n');
}

function createKeystrokeSection(element) {
  const container = document.createElement('div');
  container.className = 'field-group';

  function makeArea(label, key) {
    const section = document.createElement('div');
    section.innerHTML = `
      <label>${label}</label>
      <textarea placeholder="e.g. 4,5,6">${serializeKeystrokes(element[key])}</textarea>
    `;
    const textarea = section.querySelector('textarea');
    textarea.addEventListener('input', () => {
      element[key] = parseKeystrokeLines(textarea.value);
    });
    return section;
  }

  if ('keystroke_left' in element && 'keystroke_right' in element) {
    container.appendChild(makeArea('Left rotation', 'keystroke_left'));
    container.appendChild(makeArea('Right rotation', 'keystroke_right'));
    if ('keystroke_push' in element) {
      container.appendChild(makeArea('Push', 'keystroke_push'));
    }
  } else {
    container.appendChild(makeArea('Keystroke', 'keystroke'));
  }

  return container;
}

function parseKeystrokeLines(content) {
  return content
    .split(/\r?\n/)
    .map(line => line.trim())
    .filter(line => line.length > 0)
    .map(line => line.split(/\s*,\s*/).map(token => Number(token)).filter(Number.isFinite));
}

function renderIconCanvas(canvas, iconBytes) {
  const ctx = canvas.getContext('2d');
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.fillStyle = '#000';
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  if (!iconBytes) {
    return;
  }
  const blob = new Blob([iconBytes], { type: 'image/bmp' });
  const url = URL.createObjectURL(blob);
  const image = new Image();
  image.onload = () => {
    ctx.drawImage(image, 0, 0, canvas.width, canvas.height);
    URL.revokeObjectURL(url);
  };
  image.src = url;
}

function openIconEditor(element, previewCanvasElement) {
  currentIconTarget = element;
  const imageData = element.display_icon ? decodeBmp(element.display_icon) : null;
  iconPixels = imageData ?? new Uint8Array(22 * 22);
  iconColor = 1;
  updateEditorCanvas();
  renderPreview(encodeBmp(iconPixels));
  iconEditorModal.classList.add('open');
  updateEditorControlState();
}

function closeIconEditorModal() {
  iconEditorModal.classList.remove('open');
  currentIconTarget = null;
}

function updateEditorControlState() {
  const isTextTool = toolSelect.value === 'text';
  document.body.classList.toggle('tool-text', isTextTool);
}

function updateEditorCanvas() {
  editorCtx.clearRect(0, 0, editorCanvas.width, editorCanvas.height);
  editorCtx.fillStyle = '#000';
  editorCtx.fillRect(0, 0, editorCanvas.width, editorCanvas.height);
  const imageData = editorCtx.createImageData(22, 22);
  for (let i = 0; i < iconPixels.length; i += 1) {
    const color = iconPixels[i] ? 255 : 0;
    imageData.data[i * 4 + 0] = color;
    imageData.data[i * 4 + 1] = color;
    imageData.data[i * 4 + 2] = color;
    imageData.data[i * 4 + 3] = 255;
  }
  const offscreen = document.createElement('canvas');
  offscreen.width = 22;
  offscreen.height = 22;
  const offCtx = offscreen.getContext('2d');
  offCtx.putImageData(imageData, 0, 0);
  editorCtx.imageSmoothingEnabled = false;
  editorCtx.drawImage(offscreen, 0, 0, 220, 220);
  updatePreviewCanvas();
}

function updatePreviewCanvas() {
  const imageData = previewCtx.createImageData(22, 22);
  for (let i = 0; i < iconPixels.length; i += 1) {
    const color = iconPixels[i] ? 255 : 0;
    imageData.data[i * 4 + 0] = color;
    imageData.data[i * 4 + 1] = color;
    imageData.data[i * 4 + 2] = color;
    imageData.data[i * 4 + 3] = 255;
  }
  previewCtx.putImageData(imageData, 0, 0);
}

function setPixel(x, y, value) {
  if (x < 0 || x >= 22 || y < 0 || y >= 22) return;
  iconPixels[y * 22 + x] = value;
}

function applyBrush(x, y) {
  const size = Number(brushSizeSelect.value);
  for (let dy = 0; dy < size; dy += 1) {
    for (let dx = 0; dx < size; dx += 1) {
      setPixel(x + dx, y + dy, iconColor);
    }
  }
}

function drawText(text, x, y) {
  const canvas = document.createElement('canvas');
  canvas.width = 22;
  canvas.height = 22;
  const ctx = canvas.getContext('2d');
  ctx.fillStyle = '#000';
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  ctx.fillStyle = iconColor ? '#fff' : '#000';
  ctx.font = '18px monospace';
  ctx.textBaseline = 'top';
  ctx.fillText(text, x - 2, y - 2);
  const imageData = ctx.getImageData(0, 0, 22, 22);
  for (let py = 0; py < 22; py += 1) {
    for (let px = 0; px < 22; px += 1) {
      const idx = (py * 22 + px) * 4;
      const value = imageData.data[idx] > 128 ? 1 : 0;
      if (value) {
        setPixel(px, py, iconColor);
      }
    }
  }
}

function handlePointer(x, y) {
  const canvasRect = editorCanvas.getBoundingClientRect();
  const cx = Math.floor(((x - canvasRect.left) / canvasRect.width) * 22);
  const cy = Math.floor(((y - canvasRect.top) / canvasRect.height) * 22);
  const pixelX = clamp(cx, 0, 21);
  const pixelY = clamp(cy, 0, 21);

  if (toolSelect.value === 'text') {
    drawText(textInput.value || 'A', pixelX, pixelY);
  } else {
    applyBrush(pixelX, pixelY);
  }
  updateEditorCanvas();
}

editorCanvas.addEventListener('mousedown', (event) => {
  isMouseDown = true;
  handlePointer(event.clientX, event.clientY);
});

document.addEventListener('mouseup', () => {
  isMouseDown = false;
});

editorCanvas.addEventListener('mousemove', (event) => {
  if (!isMouseDown) return;
  handlePointer(event.clientX, event.clientY);
});

toolSelect.addEventListener('change', updateEditorControlState);
colorSelect.addEventListener('change', () => {
  iconColor = colorSelect.value === 'white' ? 1 : 0;
});

fillCanvasButton.addEventListener('click', () => {
  iconPixels.fill(iconColor);
  updateEditorCanvas();
});

clearCanvasButton.addEventListener('click', () => {
  iconPixels.fill(0);
  updateEditorCanvas();
});

closeIconEditor.addEventListener('click', closeIconEditorModal);
iconEditorModal.addEventListener('click', (event) => {
  if (event.target === iconEditorModal) closeIconEditorModal();
});

saveIconButton.addEventListener('click', () => {
  if (!currentIconTarget) return;
  const bmp = encodeBmp(iconPixels);
  currentIconTarget.display_icon = bmp;
  renderPreview(bmp);
  currentIconTarget = null;
  closeIconEditorModal();
  renderEditorGrid();
});

async function loadConfig(newConfigObject) {
  config = newConfigObject;
  renderEditorGrid();
}

async function initApp() {
  await init();
  const defaultCfg = await default_config();
  await loadConfig(defaultCfg);
}

importButton.addEventListener('click', () => configFileInput.click());
configFileInput.addEventListener('change', async (event) => {
  const file = event.target.files[0];
  if (!file) return;
  const bytes = new Uint8Array(await file.arrayBuffer());
  const loaded = await import_config_from_cbor(bytes);
  await loadConfig(loaded);
});

exportButton.addEventListener('click', async () => {
  if (!config) return;
  const bytes = await export_config_to_cbor(config);
  const blob = new Blob([bytes], { type: 'application/octet-stream' });
  const url = URL.createObjectURL(blob);
  const download = document.createElement('a');
  download.href = url;
  download.download = `${config.name || 'macropad-config'}.cbor`;
  document.body.appendChild(download);
  download.click();
  download.remove();
  URL.revokeObjectURL(url);
});

newConfigButton.addEventListener('click', async () => {
  const fresh = newConfig();
  await loadConfig(fresh);
});

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
    for (let row = 0; row < 22; row += 1) {
      const srcRow = bottomUp ? 21 - row : row;
      const rowStart = dataOffset + srcRow * paddedRowBytes;
      for (let col = 0; col < 22; col += 1) {
        const byteIndex = rowStart + (col >> 3);
        const bitIndex = 7 - (col & 7);
        const value = (bytes[byteIndex] >> bitIndex) & 1;
        pixelArray[row * 22 + col] = value;
      }
    }
  } else if (bitsPerPixel === 24) {
    for (let row = 0; row < 22; row += 1) {
      const srcRow = bottomUp ? 21 - row : row;
      const rowStart = dataOffset + srcRow * paddedRowBytes;
      for (let col = 0; col < 22; col += 1) {
        const pixelIndex = rowStart + col * 3;
        const blue = bytes[pixelIndex];
        const green = bytes[pixelIndex + 1];
        const red = bytes[pixelIndex + 2];
        const luminance = (red + green + blue) / 3;
        pixelArray[row * 22 + col] = luminance > 127 ? 1 : 0;
      }
    }
  } else {
    return null;
  }
    const srcRow = bottomUp ? 21 - row : row;
    const rowStart = dataOffset + srcRow * paddedRowBytes;
    for (let col = 0; col < 22; col += 1) {
      const byteIndex = rowStart + (col >> 3);
      const bitIndex = 7 - (col & 7);
      const value = (bytes[byteIndex] >> bitIndex) & 1;
      pixelArray[row * 22 + col] = value;
    }
  }
  return pixelArray;
}

function encodeBmp(pixels) {
  const headerSize = 14;
  const infoSize = 40;
  const paletteSize = 8;
  const rowBytes = Math.ceil(22 / 8);
  const paddedRowBytes = ((rowBytes + 3) >> 2) << 2;
  const pixelDataSize = paddedRowBytes * 22;
  const fileSize = headerSize + infoSize + paletteSize + pixelDataSize;
  const bytes = new Uint8Array(fileSize);

  bytes[0] = 0x42;
  bytes[1] = 0x4d;
  bytes[2] = fileSize & 0xff;
  bytes[3] = (fileSize >> 8) & 0xff;
  bytes[4] = (fileSize >> 16) & 0xff;
  bytes[5] = (fileSize >> 24) & 0xff;
  bytes[10] = headerSize + infoSize + paletteSize;
  bytes[14] = infoSize;
  bytes[18] = 22;
  bytes[19] = 0;
  bytes[20] = 0;
  bytes[21] = 0;
  bytes[22] = 22;
  bytes[23] = 0;
  bytes[24] = 0;
  bytes[25] = 0;
  bytes[26] = 1;
  bytes[27] = 0;
  bytes[28] = 1;
  bytes[29] = 0;
  bytes[34] = pixelDataSize & 0xff;
  bytes[35] = (pixelDataSize >> 8) & 0xff;
  bytes[36] = (pixelDataSize >> 16) & 0xff;
  bytes[37] = (pixelDataSize >> 24) & 0xff;
  bytes[38] = 0x13;
  bytes[39] = 0x0b;
  bytes[42] = 0x13;
  bytes[43] = 0x0b;

  // palette: black then white
  bytes[54] = 0;
  bytes[55] = 0;
  bytes[56] = 0;
  bytes[57] = 0;
  bytes[58] = 255;
  bytes[59] = 255;
  bytes[60] = 255;
  bytes[61] = 0;

  for (let row = 0; row < 22; row += 1) {
    const rowStart = headerSize + infoSize + paletteSize + (21 - row) * paddedRowBytes;
    for (let col = 0; col < 22; col += 1) {
      const bit = pixels[row * 22 + col] ? 1 : 0;
      const bytePos = rowStart + (col >> 3);
      const bitPos = 7 - (col & 7);
      bytes[bytePos] |= bit << bitPos;
    }
  }

  return bytes;
}

initApp().catch((error) => {
  console.error('Failed to initialize wasm module', error);
  editorGrid.textContent = 'Failed to load the web app module. Please build `webapp/wasm` for the browser.';
});
