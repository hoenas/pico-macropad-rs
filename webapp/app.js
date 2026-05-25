const KEY_CODES = [
    "NoEventIndicated", "ErrorRollOver", "POSTFail", "ErrorUndefine", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "Keyboard1", "Keyboard2", "Keyboard3", "Keyboard4", "Keyboard5", "Keyboard6", "Keyboard7", "Keyboard8", "Keyboard9", "Keyboard0", "ReturnEnter", "Escape", "DeleteBackspace", "Tab", "Space", "Minus", "Equal", "LeftBrace", "RightBrace", "Backslash", "NonUSHash", "Semicolon", "Apostrophe", "Grave", "Comma", "Dot", "ForwardSlash", "CapsLock", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12", "PrintScreen", "ScrollLock", "Pause", "Insert", "Home", "PageUp", "DeleteForward", "End", "PageDown", "RightArrow", "LeftArrow", "DownArrow", "UpArrow", "KeypadNumLockAndClear", "KeypadDivide", "KeypadMultiply", "KeypadSubtract", "KeypadAdd", "KeypadEnter", "Keypad1", "Keypad2", "Keypad3", "Keypad4", "Keypad5", "Keypad6", "Keypad7", "Keypad8", "Keypad9", "Keypad0", "KeypadDot", "NonUSBackslash", "Application", "Power", "KeypadEqual", "F13", "F14", "F15", "F16", "F17", "F18", "F19", "F20", "F21", "F22", "F23", "F24", "Execute", "Help", "Menu", "Select", "Stop", "Again", "Undo", "Cut", "Copy", "Paste", "Find", "Mute", "VolumeUp", "VolumeDown", "LockingCapsLock", "LockingNumLock", "LockingScrollLock", "KeypadComma", "KeypadEqualSign", "Kanji1", "Kanji2", "Kanji3", "Kanji4", "Kanji5", "Kanji6", "Kanji7", "Kanji8", "Kanji9", "LANG1", "LANG2", "LANG3", "LANG4", "LANG5", "LANG6", "LANG7", "LANG8", "LANG9", "AlternateErase", "SysReqAttention", "Cancel", "Clear", "Prior", "Return", "Separator", "Out", "Oper", "ClearAgain", "CrSelProps", "ExSel", "LeftControl", "LeftShift", "LeftAlt", "LeftGUI", "RightControl", "RightShift", "RightAlt", "RightGUI"
];

const buttonFields = Array.from({ length: 10 }, (_, idx) => ({ id: `button${idx}`, title: `Button ${idx + 1}` }));
const encoderFields = [
    { id: 'menu_encoder', title: 'Menu Encoder', types: ['left', 'right'] },
    { id: 'encoder1', title: 'Encoder 1', types: ['left', 'right', 'push'] },
    { id: 'encoder2', title: 'Encoder 2', types: ['left', 'right', 'push'] },
];

const elements = {
    configName: document.getElementById('config-name'),
    buttonsGrid: document.getElementById('buttons-grid'),
    encodersGrid: document.getElementById('encoders-grid'),
    ledsGrid: document.getElementById('leds-grid'),
    outputCbor: document.getElementById('output-cbor'),
    updateBtn: document.getElementById('update-btn'),
    downloadCfgBtn: document.getElementById('download-cfg'),
    downloadPackageBtn: document.getElementById('download-package'),
    loadExampleBtn: document.getElementById('load-example'),
    keycodeList: document.getElementById('keycode-list'),
    displayPreview: document.getElementById('display-preview'),
    validationErrors: document.getElementById('validation-errors'),
    feedback: document.getElementById('feedback'),
    loadCfgFile: document.getElementById('load-cfg-file'),
    loadCfgBtn: document.getElementById('load-cfg-btn'),
    iconEditorModal: document.getElementById('icon-editor-modal'),
    editorCanvas: document.getElementById('editor-canvas'),
    editorImageInput: document.getElementById('editor-image-input'),
    editorThreshold: document.getElementById('editor-threshold'),
    thresholdValue: document.getElementById('threshold-value'),
    editorInvertBtn: document.getElementById('editor-invert-btn'),
    editorConfirmBtn: document.getElementById('editor-confirm-btn'),
    editorCancelBtn: document.getElementById('editor-cancel-btn'),
    modalClose: document.querySelector('.modal-close'),
};

const iconBlobs = new Map();
let currentEditorFieldId = null;
let currentEditorImage = null;
let currentEditorInverted = false;

function normalizeSdcardPath(path) {
    const cleaned = path.trim().replace(/^\/+|^\\+/, '');
    const segments = cleaned.split(/[\\/]+/).filter(segment => segment && segment !== '.' && segment !== '..');
    if (segments.length === 0) {
        return '';
    }
    if (segments[0].toLowerCase() !== 'icons') {
        segments.unshift('icons');
    }
    return segments.join('/');
}

function utf8Encode(value) {
    return new TextEncoder().encode(value);
}

function concatUint8Arrays(arrays) {
    const length = arrays.reduce((sum, arr) => sum + arr.length, 0);
    const result = new Uint8Array(length);
    let offset = 0;
    arrays.forEach(arr => {
        result.set(arr, offset);
        offset += arr.length;
    });
    return result;
}

function encodeLength(major, length) {
    if (length < 24) {
        return new Uint8Array([((major << 5) | length)]);
    }
    if (length < 0x100) {
        return new Uint8Array([((major << 5) | 24), length]);
    }
    if (length < 0x10000) {
        const arr = new Uint8Array(3);
        arr[0] = (major << 5) | 25;
        arr[1] = (length >> 8) & 0xff;
        arr[2] = length & 0xff;
        return arr;
    }
    const arr = new Uint8Array(5);
    arr[0] = (major << 5) | 26;
    arr[1] = (length >>> 24) & 0xff;
    arr[2] = (length >>> 16) & 0xff;
    arr[3] = (length >>> 8) & 0xff;
    arr[4] = length & 0xff;
    return arr;
}

function encodeValue(value) {
    if (value === null) {
        return new Uint8Array([0xf6]);
    }
    if (typeof value === 'boolean') {
        return new Uint8Array([value ? 0xf5 : 0xf4]);
    }
    if (typeof value === 'number') {
        if (!Number.isInteger(value)) {
            throw new Error('CBOR encoder only supports integer values');
        }
        if (value >= 0) {
            return concatUint8Arrays([encodeLength(0, value)]);
        }
        return concatUint8Arrays([encodeLength(1, -value - 1)]);
    }
    if (typeof value === 'string') {
        const bytes = utf8Encode(value);
        return concatUint8Arrays([encodeLength(3, bytes.length), bytes]);
    }
    if (value instanceof Uint8Array) {
        return concatUint8Arrays([encodeLength(2, value.length), value]);
    }
    if (Array.isArray(value)) {
        const parts = [encodeLength(4, value.length)];
        value.forEach(item => parts.push(encodeValue(item)));
        return concatUint8Arrays(parts);
    }
    if (typeof value === 'object') {
        const keys = Object.keys(value);
        const parts = [encodeLength(5, keys.length)];
        keys.forEach(key => {
            parts.push(encodeValue(key));
            parts.push(encodeValue(value[key]));
        });
        return concatUint8Arrays(parts);
    }
    throw new Error('Unsupported CBOR value type');
}

function cborEncode(value) {
    return encodeValue(value);
}

function readCborLength(bytes, offset, info) {
    if (info < 24) {
        return [info, 0];
    }
    if (info === 24) {
        return [bytes[offset], 1];
    }
    if (info === 25) {
        return [(bytes[offset] << 8) | bytes[offset + 1], 2];
    }
    if (info === 26) {
        return [
            (bytes[offset] << 24) | (bytes[offset + 1] << 16) | (bytes[offset + 2] << 8) | bytes[offset + 3],
            4,
        ];
    }
    throw new Error('Unsupported CBOR length encoding');
}

function decodeCbor(bytes, offset = 0) {
    const initial = bytes[offset++];
    const major = initial >> 5;
    const info = initial & 0x1f;
    switch (major) {
        case 0: {
            const [value, lengthBytes] = readCborLength(bytes, offset, info);
            return [value, offset + lengthBytes];
        }
        case 1: {
            const [value, lengthBytes] = readCborLength(bytes, offset, info);
            return [-(value + 1), offset + lengthBytes];
        }
        case 2: {
            const [length, lengthBytes] = readCborLength(bytes, offset, info);
            const start = offset + lengthBytes;
            const end = start + length;
            return [bytes.subarray(start, end), end];
        }
        case 3: {
            const [length, lengthBytes] = readCborLength(bytes, offset, info);
            const start = offset + lengthBytes;
            const end = start + length;
            const value = new TextDecoder().decode(bytes.subarray(start, end));
            return [value, end];
        }
        case 4: {
            const [count, lengthBytes] = readCborLength(bytes, offset, info);
            offset += lengthBytes;
            const arr = [];
            for (let i = 0; i < count; i += 1) {
                const [item, nextOffset] = decodeCbor(bytes, offset);
                arr.push(item);
                offset = nextOffset;
            }
            return [arr, offset];
        }
        case 5: {
            const [count, lengthBytes] = readCborLength(bytes, offset, info);
            offset += lengthBytes;
            const obj = {};
            for (let i = 0; i < count; i += 1) {
                const [key, nextOffset] = decodeCbor(bytes, offset);
                if (typeof key !== 'string') {
                    throw new Error('CBOR map keys must be strings');
                }
                const [value, valueOffset] = decodeCbor(bytes, nextOffset);
                obj[key] = value;
                offset = valueOffset;
            }
            return [obj, offset];
        }
        case 7: {
            if (info === 20) return [false, offset];
            if (info === 21) return [true, offset];
            if (info === 22) return [null, offset];
            if (info === 23) return [undefined, offset];
            break;
        }
    }
    throw new Error('Unsupported CBOR major type: ' + major);
}

function cborDecode(buffer) {
    const bytes = buffer instanceof Uint8Array ? buffer : new Uint8Array(buffer);
    const [value, offset] = decodeCbor(bytes, 0);
    if (offset !== bytes.length) {
        throw new Error('Extra CBOR bytes detected');
    }
    return value;
}

function toHex(data) {
    return Array.from(data).map(byte => byte.toString(16).padStart(2, '0')).join(' ');
}

const CRC32_TABLE = new Uint32Array(256);
for (let i = 0; i < 256; i += 1) {
    let c = i;
    for (let j = 0; j < 8; j += 1) {
        c = (c & 1) ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    }
    CRC32_TABLE[i] = c >>> 0;
}

function crc32(data) {
    let crc = 0xffffffff;
    for (let i = 0; i < data.length; i += 1) {
        crc = (crc >>> 8) ^ CRC32_TABLE[(crc ^ data[i]) & 0xff];
    }
    return (crc ^ 0xffffffff) >>> 0;
}

function createZip(entries) {
    const localParts = [];
    const centralParts = [];
    let offset = 0;

    entries.forEach(({ name, data }) => {
        const nameBytes = utf8Encode(name);
        const crc = crc32(data);
        const localHeader = new Uint8Array(30);
        const localView = new DataView(localHeader.buffer);
        localView.setUint32(0, 0x04034b50, true);
        localView.setUint16(4, 20, true);
        localView.setUint16(6, 0, true);
        localView.setUint16(8, 0, true);
        localView.setUint16(10, 0, true);
        localView.setUint32(14, crc, true);
        localView.setUint32(18, data.length, true);
        localView.setUint32(22, data.length, true);
        localView.setUint16(26, nameBytes.length, true);
        localView.setUint16(28, 0, true);
        localParts.push(localHeader, nameBytes, data);

        const centralHeader = new Uint8Array(46);
        const centralView = new DataView(centralHeader.buffer);
        centralView.setUint32(0, 0x02014b50, true);
        centralView.setUint16(4, 20, true);
        centralView.setUint16(6, 20, true);
        centralView.setUint16(8, 0, true);
        centralView.setUint16(10, 0, true);
        centralView.setUint16(12, 0, true);
        centralView.setUint16(14, 0, true);
        centralView.setUint32(16, crc, true);
        centralView.setUint32(20, data.length, true);
        centralView.setUint32(24, data.length, true);
        centralView.setUint16(26, nameBytes.length, true);
        centralView.setUint16(28, 0, true);
        centralView.setUint16(30, 0, true);
        centralView.setUint16(32, 0, true);
        centralView.setUint16(34, 0, true);
        centralView.setUint16(36, 0, true);
        centralView.setUint32(38, 0, true);
        centralView.setUint32(42, offset, true);
        centralParts.push(centralHeader, nameBytes);

        offset += localHeader.length + nameBytes.length + data.length;
    });

    const centralSize = centralParts.reduce((sum, part) => sum + part.length, 0);
    const endRecord = new Uint8Array(22);
    const endView = new DataView(endRecord.buffer);
    endView.setUint32(0, 0x06054b50, true);
    endView.setUint16(4, 0, true);
    endView.setUint16(6, 0, true);
    endView.setUint16(8, entries.length, true);
    endView.setUint16(10, entries.length, true);
    endView.setUint32(12, centralSize, true);
    endView.setUint32(16, offset, true);
    endView.setUint16(20, 0, true);

    return new Blob([...localParts, ...centralParts, endRecord], { type: 'application/zip' });
}

function createBmpBytes(width, height, pixels) {
    const rowBytes = Math.ceil(width / 32) * 4;
    const pixelDataSize = rowBytes * height;
    const headerSize = 14 + 40 + 8;
    const buffer = new ArrayBuffer(headerSize + pixelDataSize);
    const view = new DataView(buffer);
    let offset = 0;

    view.setUint8(offset++, 0x42);
    view.setUint8(offset++, 0x4d);
    view.setUint32(offset, headerSize + pixelDataSize, true);
    offset += 4;
    view.setUint16(offset, 0, true);
    offset += 2;
    view.setUint16(offset, 0, true);
    offset += 2;
    view.setUint32(offset, headerSize, true);
    offset += 4;

    view.setUint32(offset, 40, true);
    offset += 4;
    view.setInt32(offset, width, true);
    offset += 4;
    view.setInt32(offset, height, true);
    offset += 4;
    view.setUint16(offset, 1, true);
    offset += 2;
    view.setUint16(offset, 1, true);
    offset += 2;
    view.setUint32(offset, 0, true);
    offset += 4;
    view.setUint32(offset, pixelDataSize, true);
    offset += 4;
    view.setInt32(offset, 2835, true);
    offset += 4;
    view.setInt32(offset, 2835, true);
    offset += 4;
    view.setUint32(offset, 2, true);
    offset += 4;
    view.setUint32(offset, 0, true);
    offset += 4;

    view.setUint8(offset++, 0x00);
    view.setUint8(offset++, 0x00);
    view.setUint8(offset++, 0x00);
    view.setUint8(offset++, 0x00);
    view.setUint8(offset++, 0xff);
    view.setUint8(offset++, 0xff);
    view.setUint8(offset++, 0xff);
    view.setUint8(offset++, 0x00);

    const pixelOffset = headerSize;
    for (let row = height - 1; row >= 0; row -= 1) {
        let byte = 0;
        let bitPos = 7;
        let outOffset = pixelOffset + (height - 1 - row) * rowBytes;

        for (let x = 0; x < width; x += 1) {
            if (pixels[row * width + x]) {
                byte |= 1 << bitPos;
            }
            bitPos -= 1;
            if (bitPos < 0) {
                view.setUint8(outOffset++, byte);
                byte = 0;
                bitPos = 7;
            }
        }

        if (bitPos !== 7) {
            view.setUint8(outOffset++, byte);
        }

        while (outOffset < pixelOffset + (height - 1 - row) * rowBytes + rowBytes) {
            view.setUint8(outOffset++, 0);
        }
    }

    return new Uint8Array(buffer);
}

function createBmpBlob(width, height, pixels) {
    return new Blob([createBmpBytes(width, height, pixels)], { type: 'image/bmp' });
}

function convertIconImage(img, threshold = 128, invert = false) {
    const canvas = document.createElement('canvas');
    canvas.width = 22;
    canvas.height = 22;
    const ctx = canvas.getContext('2d');
    if (!ctx) {
        return null;
    }

    ctx.clearRect(0, 0, 22, 22);
    ctx.drawImage(img, 0, 0, 22, 22);
    const imageData = ctx.getImageData(0, 0, 22, 22);
    const pixels = new Uint8Array(484);
    const data = imageData.data;

    for (let i = 0; i < data.length; i += 4) {
        const r = data[i];
        const g = data[i + 1];
        const b = data[i + 2];
        const gray = 0.299 * r + 0.587 * g + 0.114 * b;
        let pixel = gray >= threshold ? 1 : 0;
        if (invert) {
            pixel = 1 - pixel;
        }
        const value = pixel ? 255 : 0;
        data[i] = value;
        data[i + 1] = value;
        data[i + 2] = value;
        pixels[i / 4] = pixel;
    }

    ctx.putImageData(imageData, 0, 0);
    const bytes = createBmpBytes(22, 22, pixels);
    return { blob: new Blob([bytes], { type: 'image/bmp' }), bytes, pixels };
}

function setFieldIcon(fieldId, file) {
    const reader = new FileReader();
    reader.onload = () => {
        const image = new Image();
        image.onload = () => {
            currentEditorFieldId = fieldId;
            currentEditorImage = image;
            currentEditorInverted = false;
            elements.editorThreshold.value = 128;
            elements.thresholdValue.textContent = 128;
            elements.editorImageInput.value = '';
            drawEditorPreview();
            openIconEditor();
        };
        image.src = reader.result;
    };
    reader.readAsDataURL(file);
}

function openIconEditor() {
    elements.iconEditorModal.classList.remove('hidden');
}

function closeIconEditor() {
    elements.iconEditorModal.classList.add('hidden');
    currentEditorFieldId = null;
    currentEditorImage = null;
    currentEditorInverted = false;
}

function drawEditorPreview() {
    if (!currentEditorImage || !elements.editorCanvas) {
        return;
    }
    const threshold = parseInt(elements.editorThreshold.value, 10);
    const iconData = convertIconImage(currentEditorImage, threshold, currentEditorInverted);
    if (!iconData) {
        return;
    }

    const ctx = elements.editorCanvas.getContext('2d');
    if (!ctx) {
        return;
    }

    const img = new Image();
    img.onload = () => {
        ctx.drawImage(img, 0, 0, 22, 22);
    };
    img.src = URL.createObjectURL(iconData.blob);
}

function confirmIconEdit() {
    if (!currentEditorFieldId || !currentEditorImage) {
        return;
    }

    const threshold = parseInt(elements.editorThreshold.value, 10);
    const iconData = convertIconImage(currentEditorImage, threshold, currentEditorInverted);
    if (!iconData) {
        showFeedback('Failed to convert icon image.', 'error');
        closeIconEditor();
        return;
    }

    const fileName = `${currentEditorFieldId}.bmp`;
    const defaultPath = normalizeSdcardPath(`icons/${fileName}`);
    const pathInput = document.getElementById(`${currentEditorFieldId}-icon-path`);
    if (pathInput instanceof HTMLInputElement && !pathInput.value.trim()) {
        pathInput.value = defaultPath;
    }

    iconBlobs.set(currentEditorFieldId, { 
        blob: iconData.blob, 
        bytes: iconData.bytes, 
        pixels: iconData.pixels, 
        name: fileName 
    });
    setFieldIconPreview(currentEditorFieldId, iconData.blob);
    const downloadBtn = document.getElementById(`${currentEditorFieldId}-icon-download`);
    if (downloadBtn instanceof HTMLButtonElement) {
        downloadBtn.disabled = false;
    }

    closeIconEditor();
    updateOutput();
    showFeedback('Icon updated successfully.', 'success');
}

function downloadFieldIcon(fieldId) {
    const icon = iconBlobs.get(fieldId);
    if (!icon) {
        return;
    }
    const url = URL.createObjectURL(icon.blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = icon.name;
    a.click();
    URL.revokeObjectURL(url);
}

function createTextInput(id, labelText, placeholder = '') {
    return `
        <label for="${id}">
            ${labelText}
            <input id="${id}" type="text" placeholder="${placeholder}" />
        </label>
    `;
}

function createButtonCard(field) {
    return `
        <div class="card">
            <h3>${field.title}</h3>
            ${createTextInput(`${field.id}-text`, 'Display text', 'Label or name')}
            <label class="icon-input-label">
                Choose icon
                <input id="${field.id}-icon-file" class="field-icon-file" type="file" accept="image/*" />
            </label>
            <div class="button-icon-row">
                <canvas id="${field.id}-icon-preview" class="button-icon-preview" width="22" height="22"></canvas>
                <button id="${field.id}-icon-edit" class="icon-edit-btn" type="button">Edit</button>
                <button id="${field.id}-icon-download" class="icon-download" type="button" disabled>Download</button>
            </div>
            <label for="${field.id}-keystrokes">
                Keystrokes
                <textarea id="${field.id}-keystrokes" rows="4" placeholder="A\nLeftControl,C"></textarea>
            </label>
            <small>One chord per line. Comma-separated key names.</small>
        </div>
    `;
}

function createEncoderCard(field) {
    const sections = field.types.map(type => `
        <label for="${field.id}-${type}-keystrokes">
            Keystrokes ${type}
            <textarea id="${field.id}-${type}-keystrokes" rows="4" placeholder="LeftControl,C\nA"></textarea>
        </label>
    `).join('');

    return `
        <div class="card">
            <h3>${field.title}</h3>
            ${createTextInput(`${field.id}-text`, 'Display text', 'Label or name')}
            <label class="icon-input-label">
                Choose icon
                <input id="${field.id}-icon-file" class="field-icon-file" type="file" accept="image/*" />
            </label>
            <div class="button-icon-row">
                <canvas id="${field.id}-icon-preview" class="button-icon-preview" width="22" height="22"></canvas>
                <button id="${field.id}-icon-edit" class="icon-edit-btn" type="button">Edit</button>
                <button id="${field.id}-icon-download" class="icon-download" type="button" disabled>Download</button>
            </div>
            ${sections}
            <small>One chord per line. Comma-separated key names.</small>
        </div>
    `;
}

function createLedCard(index) {
    return `
        <div class="card">
            <h3>LED ${index + 1}</h3>
            <label>
                Color
                <input id="led-${index}" type="color" value="#ffffff" />
            </label>
        </div>
    `;
}

function parseKeystrokes(text) {
    return text
        .split('\n')
        .map(line => line.trim())
        .filter(line => line.length > 0)
        .map(line => line.split(',').map(key => key.trim()).filter(Boolean));
}

function formatKeystrokes(chords) {
    if (!Array.isArray(chords)) {
        return '';
    }
    return chords.map(chord => (Array.isArray(chord) ? chord.join(',') : '')).join('\n');
}

function buildConfig() {
    const config = {
        name: elements.configName.value.trim(),
    };

    buttonFields.forEach(field => {
        const text = document.getElementById(`${field.id}-text`).value.trim();
        const keystrokes = parseKeystrokes(document.getElementById(`${field.id}-keystrokes`).value);

        const buttonConfig = {
            display_text: text,
            keystroke: keystrokes,
        };
        const icon = iconBlobs.get(field.id);
        if (icon) {
            buttonConfig.display_icon = icon.bytes;
        }
        config[field.id] = buttonConfig;
    });

    encoderFields.forEach(field => {
        const text = document.getElementById(`${field.id}-text`).value.trim();
        const encoderConfig = {
            display_text: text,
        };

        const icon = iconBlobs.get(field.id);
        if (icon) {
            encoderConfig.display_icon = icon.bytes;
        }

        field.types.forEach(type => {
            encoderConfig[`keystroke_${type}`] = parseKeystrokes(document.getElementById(`${field.id}-${type}-keystrokes`).value);
        });

        config[field.id] = encoderConfig;
    });

    config.leds = Array.from({ length: 8 }, (_, idx) => {
        const colorValue = document.getElementById(`led-${idx}`).value;
        const r = parseInt(colorValue.slice(1, 3), 16);
        const g = parseInt(colorValue.slice(3, 5), 16);
        const b = parseInt(colorValue.slice(5, 7), 16);
        return { r, g, b };
    });

    return config;
}

function updateDisplayPreview(config) {
    let html = `<div class="display-name">${config.name || 'Unnamed'}</div>`;
    html += '<div class="display-buttons">';

    buttonFields.forEach(field => {
        const btn = config[field.id];
        const icon = iconBlobs.get(field.id);
        if (icon) {
            html += `<div class="display-button"><canvas class="display-icon-canvas" id="display-preview-${field.id}" width="22" height="22"></canvas></div>`;
        } else {
            html += `<div class="display-button">${btn.display_text || ''}</div>`;
        }
    });

    html += '</div>';
    elements.displayPreview.innerHTML = html;

    buttonFields.forEach(field => {
        const icon = iconBlobs.get(field.id);
        if (icon) {
            const canvas = document.getElementById(`display-preview-${field.id}`);
            if (canvas instanceof HTMLCanvasElement) {
                drawDisplayIcon(canvas, icon);
            }
        }
    });
}

function drawDisplayIcon(canvas, icon) {
    const ctx = canvas.getContext('2d');
    if (!ctx) {
        return;
    }
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    if (!icon || !icon.blob) {
        return;
    }
    const img = new Image();
    img.onload = () => {
        ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
        URL.revokeObjectURL(img.src);
    };
    img.src = URL.createObjectURL(icon.blob);
}

function validateConfig(config) {
    const errors = [];

    if (!config.name || typeof config.name !== 'string') {
        errors.push('Config name must be a non-empty string.');
    }

    buttonFields.forEach((field, index) => {
        const btn = config[field.id];
        if (!btn || typeof btn !== 'object') {
            errors.push(`Button ${index + 1}: Invalid structure.`);
            return;
        }
        if (typeof btn.display_text !== 'string') {
            errors.push(`Button ${index + 1}: display_text must be a string.`);
        }
        if (btn.display_icon != null && !(btn.display_icon instanceof Uint8Array || (Array.isArray(btn.display_icon) && btn.display_icon.every(byte => typeof byte === 'number' && Number.isInteger(byte) && byte >= 0 && byte <= 255)))) {
            errors.push(`Button ${index + 1}: display_icon must be a byte array or Uint8Array.`);
        }
        if (!Array.isArray(btn.keystroke)) {
            errors.push(`Button ${index + 1}: keystroke must be an array.`);
        } else {
            btn.keystroke.forEach((chord, chordIndex) => {
                if (!Array.isArray(chord)) {
                    errors.push(`Button ${index + 1}: keystroke[${chordIndex}] must be an array.`);
                } else {
                    chord.forEach(key => {
                        if (!KEY_CODES.includes(key)) {
                            errors.push(`Button ${index + 1}: Invalid key '${key}' in keystroke.`);
                        }
                    });
                }
            });
        }
    });

    encoderFields.forEach(field => {
        const enc = config[field.id];
        if (!enc || typeof enc !== 'object') {
            errors.push(`${field.title}: Invalid structure.`);
            return;
        }
        if (typeof enc.display_text !== 'string') {
            errors.push(`${field.title}: display_text must be a string.`);
        }
        if (enc.display_icon != null && !(enc.display_icon instanceof Uint8Array || (Array.isArray(enc.display_icon) && enc.display_icon.every(byte => typeof byte === 'number' && Number.isInteger(byte) && byte >= 0 && byte <= 255)))) {
            errors.push(`${field.title}: display_icon must be a byte array or Uint8Array.`);
        }
        field.types.forEach(type => {
            const dir = enc[`keystroke_${type}`];
            if (!Array.isArray(dir)) {
                errors.push(`${field.title}: keystroke_${type} must be an array.`);
            } else {
                dir.forEach((chord, chordIndex) => {
                    if (!Array.isArray(chord)) {
                        errors.push(`${field.title}: keystroke_${type}[${chordIndex}] must be an array.`);
                    } else {
                        chord.forEach(key => {
                            if (!KEY_CODES.includes(key)) {
                                errors.push(`${field.title}: Invalid key '${key}' in keystroke_${type}.`);
                            }
                        });
                    }
                });
            }
        });
    });

    if (!Array.isArray(config.leds) || config.leds.length !== 8) {
        errors.push('LEDs must be an array of 8 objects.');
    } else {
        config.leds.forEach((led, idx) => {
            if (typeof led !== 'object' || led === null) {
                errors.push(`LED ${idx + 1}: Must be an object.`);
            } else {
                ['r', 'g', 'b'].forEach(color => {
                    if (typeof led[color] !== 'number' || led[color] < 0 || led[color] > 255 || !Number.isInteger(led[color])) {
                        errors.push(`LED ${idx + 1}: ${color} must be an integer 0-255.`);
                    }
                });
            }
        });
    }

    return errors;
}

function showFeedback(message, type = 'info') {
    elements.feedback.textContent = message;
    elements.feedback.className = `feedback ${type}`;
    setTimeout(() => {
        elements.feedback.textContent = '';
        elements.feedback.className = 'feedback';
    }, 4000);
}

function downloadCfg() {
    const config = buildConfig();
    const errors = validateConfig(config);
    if (errors.length > 0) {
        elements.validationErrors.innerHTML = '<ul>' + errors.map(error => `<li>${error}</li>`).join('') + '</ul>';
        showFeedback('Fix config errors before downloading.', 'error');
        return;
    }
    elements.validationErrors.innerHTML = '';

    const content = cborEncode(config);
    const blob = new Blob([content], { type: 'application/octet-stream' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    const filename = sanitizeFilename(elements.configName.value.trim() || 'macro_config');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
    showFeedback(`Config saved as ${filename}`, 'success');
}

function downloadPackage() {
    const config = buildConfig();
    const errors = validateConfig(config);
    if (errors.length > 0) {
        elements.validationErrors.innerHTML = '<ul>' + errors.map(error => `<li>${error}</li>`).join('') + '</ul>';
        showFeedback('Fix config errors before exporting the package.', 'error');
        return;
    }
    elements.validationErrors.innerHTML = '';

    const entries = [];
    const configName = sanitizeFilename(elements.configName.value.trim() || 'macro_config');
    entries.push({ name: configName, data: cborEncode(config) });

    const iconPaths = new Map();

    [...buttonFields, ...encoderFields].forEach(field => {
        const icon = iconBlobs.get(field.id);
        if (icon) {
            const path = normalizeSdcardPath(`icons/${field.id}.bmp`);
            if (!iconPaths.has(path)) {
                iconPaths.set(path, icon.blob);
            }
        }
    });

    iconPaths.forEach((blob, path) => {
        entries.push({ name: path, data: new Uint8Array(blobToArrayBuffer(blob)) });
    });

    const zip = createZip(entries);
    const url = URL.createObjectURL(zip);
    const a = document.createElement('a');
    a.href = url;
    a.download = elements.configName.value.trim() ? `${elements.configName.value.trim()}-sdcard.zip` : 'macro_config-sdcard.zip';
    a.click();
    URL.revokeObjectURL(url);
    showFeedback('SD card package exported.', 'success');
}

function blobToArrayBuffer(blob) {
    const reader = new FileReaderSync();
    return reader.readAsArrayBuffer(blob);
}

function sanitizeFilename(name) {
    let base = name.replace(/[^a-zA-Z0-9_]/g, '').toUpperCase();
    if (base.length === 0) base = 'CONFIG';
    base = base.substring(0, 8);
    return `${base}.CFG`;
}

function loadCfg() {
    const file = elements.loadCfgFile.files?.[0];
    if (!file) {
        showFeedback('Please select a config file first.', 'error');
        return;
    }
    const reader = new FileReader();
    reader.onload = () => {
        try {
            const config = cborDecode(reader.result);
            loadConfigIntoForm(config);
            updateOutput();
            showFeedback(`Config loaded: ${file.name}`, 'success');
        } catch (error) {
            showFeedback(`Invalid config file: ${error.message}`, 'error');
        }
    };
    reader.readAsArrayBuffer(file);
}

function loadConfigIntoForm(config) {
    iconBlobs.clear();
    elements.configName.value = config.name || '';

    buttonFields.forEach(field => {
        const btn = config[field.id] || {};
        document.getElementById(`${field.id}-text`).value = btn.display_text || '';

        const btnIconBytes = btn.display_icon instanceof Uint8Array ? btn.display_icon : Array.isArray(btn.display_icon) ? Uint8Array.from(btn.display_icon) : null;
        if (btnIconBytes && btnIconBytes.length > 0) {
            const bmpBlob = new Blob([btnIconBytes], { type: 'image/bmp' });
            iconBlobs.set(field.id, { blob: bmpBlob, bytes: btnIconBytes, name: `${field.id}.bmp` });
            setFieldIconPreview(field.id, bmpBlob);
            const downloadBtn = document.getElementById(`${field.id}-icon-download`);
            if (downloadBtn instanceof HTMLButtonElement) {
                downloadBtn.disabled = false;
            }
        } else {
            setFieldIconPreview(field.id, null);
            const downloadBtn = document.getElementById(`${field.id}-icon-download`);
            if (downloadBtn instanceof HTMLButtonElement) {
                downloadBtn.disabled = true;
            }
        }

        document.getElementById(`${field.id}-keystrokes`).value = formatKeystrokes(btn.keystroke || []);
    });

    encoderFields.forEach(field => {
        const enc = config[field.id] || {};
        document.getElementById(`${field.id}-text`).value = enc.display_text || '';

        const encIconBytes = enc.display_icon instanceof Uint8Array ? enc.display_icon : Array.isArray(enc.display_icon) ? Uint8Array.from(enc.display_icon) : null;
        if (encIconBytes && encIconBytes.length > 0) {
            const bmpBlob = new Blob([encIconBytes], { type: 'image/bmp' });
            iconBlobs.set(field.id, { blob: bmpBlob, bytes: encIconBytes, name: `${field.id}.bmp` });
            setFieldIconPreview(field.id, bmpBlob);
            const downloadBtn = document.getElementById(`${field.id}-icon-download`);
            if (downloadBtn instanceof HTMLButtonElement) {
                downloadBtn.disabled = false;
            }
        } else {
            setFieldIconPreview(field.id, null);
            const downloadBtn = document.getElementById(`${field.id}-icon-download`);
            if (downloadBtn instanceof HTMLButtonElement) {
                downloadBtn.disabled = true;
            }
        }

        field.types.forEach(type => {
            document.getElementById(`${field.id}-${type}-keystrokes`).value = formatKeystrokes(enc[`keystroke_${type}`] || []);
        });
    });

    if (Array.isArray(config.leds)) {
        config.leds.forEach((led, idx) => {
            if (led && typeof led === 'object') {
                const r = Math.max(0, Math.min(255, led.r || 0));
                const g = Math.max(0, Math.min(255, led.g || 0));
                const b = Math.max(0, Math.min(255, led.b || 0));
                document.getElementById(`led-${idx}`).value = `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`;
            }
        });
    }
}

function setFieldIconPreview(fieldId, blob) {
    const preview = document.getElementById(`${fieldId}-icon-preview`);
    if (!(preview instanceof HTMLCanvasElement)) {
        return;
    }
    const ctx = preview.getContext('2d');
    if (!ctx) {
        return;
    }
    ctx.clearRect(0, 0, preview.width, preview.height);
    if (!blob) {
        return;
    }
    const img = new Image();
    img.onload = () => {
        ctx.drawImage(img, 0, 0, preview.width, preview.height);
    };
    img.src = URL.createObjectURL(blob);
}

function updateOutput() {
    const config = buildConfig();
    const errors = validateConfig(config);
    if (errors.length > 0) {
        elements.validationErrors.innerHTML = '<ul>' + errors.map(error => `<li>${error}</li>`).join('') + '</ul>';
    } else {
        elements.validationErrors.innerHTML = '';
    }
    const content = cborEncode(config);
    elements.outputCbor.value = `CBOR (${content.length} bytes):\n${toHex(content)}`;
    updateDisplayPreview(config);
}

function render() {
    elements.buttonsGrid.innerHTML = buttonFields.map(createButtonCard).join('');
    elements.encodersGrid.innerHTML = encoderFields.map(createEncoderCard).join('');
    elements.ledsGrid.innerHTML = Array.from({ length: 8 }, (_, idx) => createLedCard(idx)).join('');
    elements.keycodeList.textContent = KEY_CODES.join(', ');
}

function loadExampleConfig() {
    elements.configName.value = 'example';
    buttonFields.forEach((field, idx) => {
        const char = String.fromCharCode(65 + idx);
        document.getElementById(`${field.id}-text`).value = char;
        document.getElementById(`${field.id}-keystrokes`).value = `${char}`;
        setFieldIconPreview(field.id, null);
        const downloadBtn = document.getElementById(`${field.id}-icon-download`);
        if (downloadBtn instanceof HTMLButtonElement) {
            downloadBtn.disabled = true;
        }
    });

    encoderFields.forEach(field => {
        document.getElementById(`${field.id}-text`).value = field.title;
        field.types.forEach(type => {
            document.getElementById(`${field.id}-${type}-keystrokes`).value = type === 'left' ? 'LeftControl,C' : type === 'right' ? 'LeftControl,V' : 'LeftControl,V';
        });
        setFieldIconPreview(field.id, null);
        const downloadBtn = document.getElementById(`${field.id}-icon-download`);
        if (downloadBtn instanceof HTMLButtonElement) {
            downloadBtn.disabled = true;
        }
    });

    Array.from({ length: 8 }, (_, idx) => {
        document.getElementById(`led-${idx}`).value = ['#ff0000', '#00ff00', '#0000ff', '#ffff00', '#ff00ff', '#00ffff', '#ffffff', '#808080'][idx];
    });

    iconBlobs.clear();
    updateOutput();
}

function collectIconFile(fieldId) {
    const input = document.getElementById(`${fieldId}-icon-file`);
    if (!(input instanceof HTMLInputElement) || !input.files?.[0]) {
        return;
    }
    setFieldIcon(fieldId, input.files[0]);
}

render();
loadExampleConfig();

// Modal event listeners
elements.modalClose.addEventListener('click', closeIconEditor);
elements.iconEditorModal.addEventListener('click', (event) => {
    if (event.target === elements.iconEditorModal) {
        closeIconEditor();
    }
});

elements.editorThreshold.addEventListener('input', (event) => {
    const value = event.target.value;
    elements.thresholdValue.textContent = value;
    drawEditorPreview();
});

elements.editorInvertBtn.addEventListener('click', () => {
    currentEditorInverted = !currentEditorInverted;
    elements.editorInvertBtn.textContent = currentEditorInverted ? 'Invert Colors (on)' : 'Invert Colors';
    drawEditorPreview();
});

elements.editorImageInput.addEventListener('change', (event) => {
    const file = event.target.files?.[0];
    if (file) {
        const reader = new FileReader();
        reader.onload = () => {
            const image = new Image();
            image.onload = () => {
                currentEditorImage = image;
                currentEditorInverted = false;
                elements.editorThreshold.value = 128;
                elements.thresholdValue.textContent = 128;
                elements.editorInvertBtn.textContent = 'Invert Colors';
                drawEditorPreview();
            };
            image.src = reader.result;
        };
        reader.readAsDataURL(file);
    }
});

elements.editorConfirmBtn.addEventListener('click', confirmIconEdit);
elements.editorCancelBtn.addEventListener('click', closeIconEditor);

elements.updateBtn.addEventListener('click', updateOutput);
elements.downloadCfgBtn.addEventListener('click', downloadCfg);
elements.downloadPackageBtn.addEventListener('click', downloadPackage);
elements.loadExampleBtn.addEventListener('click', loadExampleConfig);
elements.loadCfgBtn.addEventListener('click', loadCfg);

document.addEventListener('change', event => {
    const target = event.target;
    if (!(target instanceof HTMLElement)) {
        return;
    }
    if (target.classList.contains('field-icon-file')) {
        const fieldId = target.id.replace(/-icon-file$/, '');
        collectIconFile(fieldId);
    }
});

document.addEventListener('click', event => {
    const target = event.target;
    if (!(target instanceof HTMLElement)) {
        return;
    }
    if (target.classList.contains('icon-edit-btn')) {
        const fieldId = target.id.replace(/-icon-edit$/, '');
        const icon = iconBlobs.get(fieldId);
        if (icon) {
            // Load the existing icon into the editor
            const blob = icon.blob;
            const img = new Image();
            img.onload = () => {
                currentEditorFieldId = fieldId;
                currentEditorImage = img;
                currentEditorInverted = false;
                elements.editorThreshold.value = 128;
                elements.thresholdValue.textContent = 128;
                elements.editorImageInput.value = '';
                elements.editorInvertBtn.textContent = 'Invert Colors';
                drawEditorPreview();
                openIconEditor();
            };
            img.src = URL.createObjectURL(blob);
        } else {
            showFeedback('Please load an icon first.', 'info');
        }
    }
});

[...document.querySelectorAll('input, textarea')].forEach(el => {
    el.addEventListener('input', updateOutput);
});
