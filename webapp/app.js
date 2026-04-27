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
    outputJson: document.getElementById('output-json'),
    updateJsonBtn: document.getElementById('update-json'),
    copyJsonBtn: document.getElementById('copy-json'),
    downloadJsonBtn: document.getElementById('download-json'),
    loadExampleBtn: document.getElementById('load-example'),
    keycodeList: document.getElementById('keycode-list'),
    iconFile: document.getElementById('icon-file'),
    convertIconBtn: document.getElementById('convert-icon'),
    downloadIconBtn: document.getElementById('download-icon'),
    iconCanvas: document.getElementById('icon-canvas'),
    displayPreview: document.getElementById('display-preview'),
    validationErrors: document.getElementById('validation-errors'),
    loadJsonFile: document.getElementById('load-json-file'),
    loadJsonBtn: document.getElementById('load-json-btn'),
};

const buttonIconBlobs = new Map();

let currentIconBlob = null;

function createBmpBlob(width, height, pixels) {
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

    return new Blob([buffer], { type: 'image/bmp' });
}

function convertIconImage(img) {
    const ctx = elements.iconCanvas.getContext('2d');
    if (!ctx) {
        return null;
    }
    ctx.clearRect(0, 0, 20, 20);
    ctx.drawImage(img, 0, 0, 20, 20);
    const imageData = ctx.getImageData(0, 0, 20, 20);
    const pixels = new Uint8Array(20 * 20);
    const data = imageData.data;

    for (let i = 0; i < data.length; i += 4) {
        const r = data[i];
        const g = data[i + 1];
        const b = data[i + 2];
        const gray = 0.299 * r + 0.587 * g + 0.114 * b;
        const value = gray >= 128 ? 255 : 0;
        data[i] = value;
        data[i + 1] = value;
        data[i + 2] = value;
        pixels[i / 4] = gray >= 128 ? 1 : 0;
    }

    ctx.putImageData(imageData, 0, 0);
    return createBmpBlob(20, 20, pixels);
}

function updateIconFromFile() {
    const file = elements.iconFile.files?.[0];
    if (!file) {
        return;
    }
    const reader = new FileReader();
    reader.onload = () => {
        const image = new Image();
        image.onload = () => {
            currentIconBlob = convertIconImage(image);
            elements.downloadIconBtn.disabled = !currentIconBlob;
        };
        image.src = reader.result;
    };
    reader.readAsDataURL(file);
}

function downloadIcon() {
    if (!currentIconBlob) {
        return;
    }
    const url = URL.createObjectURL(currentIconBlob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'icon.bmp';
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

function createDatalist(id) {
    return `
        <datalist id="${id}">
            ${KEY_CODES.map(code => `<option value="${code}"></option>`).join('')}
        </datalist>
    `;
}

function createButtonCard(field) {
    const html = `
        <div class="card">
            <h3>${field.title}</h3>
            ${createTextInput(`${field.id}-text`, 'Display text', 'Label or name')}
            ${createTextInput(`${field.id}-icon`, 'Display icon (optional)', 'Icon name or empty')}
            <label>
                Upload icon image
                <input id="${field.id}-icon-file" class="button-icon-file" type="file" accept="image/*" />
            </label>
            <div class="button-icon-row">
                <canvas id="${field.id}-icon-preview" class="button-icon-preview" width="20" height="20"></canvas>
                <button id="${field.id}-icon-download" class="icon-download" type="button" disabled>Download icon BMP</button>
            </div>
            <label for="${field.id}-keystrokes">
                Keystrokes
                <textarea id="${field.id}-keystrokes" rows="5" placeholder="A\nLeftControl,C"></textarea>
            </label>
            <small>One chord per line. Comma-separated key names.</small>
        </div>
    `;
    return html;
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
            ${createTextInput(`${field.id}-icon`, 'Display icon (optional)', 'Icon name or empty')}
            ${sections}
            <small>One chord per line. Comma-separated key names.</small>
        </div>
    `;
}

function createLedCard(index) {
    return `
        <div class="card">
            <h3>LED ${index}</h3>
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

function buildConfig() {
    const config = {
        name: elements.configName.value.trim(),
    };

    buttonFields.forEach(field => {
        const text = document.getElementById(`${field.id}-text`).value.trim();
        const icon = document.getElementById(`${field.id}-icon`).value.trim();
        const keystrokes = parseKeystrokes(document.getElementById(`${field.id}-keystrokes`).value);
        config[field.id] = {
            display_text: text,
            display_icon: icon === '' ? null : icon,
            keystroke: keystrokes,
        };
    });

    encoderFields.forEach(field => {
        const text = document.getElementById(`${field.id}-text`).value.trim();
        const icon = document.getElementById(`${field.id}-icon`).value.trim();
        const encoderConfig = {
            display_text: text,
            display_icon: icon === '' ? null : icon,
        };

        field.types.forEach(type => {
            encoderConfig[`keystroke_${type}`] = parseKeystrokes(
                document.getElementById(`${field.id}-${type}-keystrokes`).value,
            );
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
    let html = `<div class="display-name">${config.name}</div>`;
    html += '<div class="display-buttons">';
    buttonFields.forEach(field => {
        const btn = config[field.id];
        html += `<div class="display-button">${btn.display_text || ''}</div>`;
    });
    html += '</div>';
    elements.displayPreview.innerHTML = html;
}

function validateConfig(config) {
    const errors = [];

    // Check name
    if (!config.name || typeof config.name !== 'string') {
        errors.push('Config name must be a non-empty string.');
    }

    // Check buttons
    for (let i = 0; i < 10; i++) {
        const btn = config[`button${i}`];
        if (!btn || typeof btn !== 'object') {
            errors.push(`Button ${i + 1}: Invalid structure.`);
            continue;
        }
        if (typeof btn.display_text !== 'string') {
            errors.push(`Button ${i + 1}: display_text must be a string.`);
        }
        if (btn.display_icon !== null && typeof btn.display_icon !== 'string') {
            errors.push(`Button ${i + 1}: display_icon must be a string or null.`);
        }
        if (!Array.isArray(btn.keystroke)) {
            errors.push(`Button ${i + 1}: keystroke must be an array.`);
        } else {
            btn.keystroke.forEach((chord, idx) => {
                if (!Array.isArray(chord)) {
                    errors.push(`Button ${i + 1}: keystroke[${idx}] must be an array.`);
                } else {
                    chord.forEach(key => {
                        if (!KEY_CODES.includes(key)) {
                            errors.push(`Button ${i + 1}: Invalid key '${key}' in keystroke.`);
                        }
                    });
                }
            });
        }
    }

    // Check menu_encoder
    const menuEnc = config.menu_encoder;
    if (!menuEnc || typeof menuEnc !== 'object') {
        errors.push('Menu Encoder: Invalid structure.');
    } else {
        if (typeof menuEnc.display_text !== 'string') {
            errors.push('Menu Encoder: display_text must be a string.');
        }
        if (menuEnc.display_icon !== null && typeof menuEnc.display_icon !== 'string') {
            errors.push('Menu Encoder: display_icon must be a string or null.');
        }
        ['keystroke_left', 'keystroke_right'].forEach(dir => {
            if (!Array.isArray(menuEnc[dir])) {
                errors.push(`Menu Encoder: ${dir} must be an array.`);
            } else {
                menuEnc[dir].forEach((chord, idx) => {
                    if (!Array.isArray(chord)) {
                        errors.push(`Menu Encoder: ${dir}[${idx}] must be an array.`);
                    } else {
                        chord.forEach(key => {
                            if (!KEY_CODES.includes(key)) {
                                errors.push(`Menu Encoder: Invalid key '${key}' in ${dir}.`);
                            }
                        });
                    }
                });
            }
        });
    }

    // Check encoder1 and encoder2
    ['encoder1', 'encoder2'].forEach(encName => {
        const enc = config[encName];
        if (!enc || typeof enc !== 'object') {
            errors.push(`${encName}: Invalid structure.`);
            return;
        }
        if (typeof enc.display_text !== 'string') {
            errors.push(`${encName}: display_text must be a string.`);
        }
        if (enc.display_icon !== null && typeof enc.display_icon !== 'string') {
            errors.push(`${encName}: display_icon must be a string or null.`);
        }
        ['keystroke_left', 'keystroke_right', 'keystroke_push'].forEach(dir => {
            if (!Array.isArray(enc[dir])) {
                errors.push(`${encName}: ${dir} must be an array.`);
            } else {
                enc[dir].forEach((chord, idx) => {
                    if (!Array.isArray(chord)) {
                        errors.push(`${encName}: ${dir}[${idx}] must be an array.`);
                    } else {
                        chord.forEach(key => {
                            if (!KEY_CODES.includes(key)) {
                                errors.push(`${encName}: Invalid key '${key}' in ${dir}.`);
                            }
                        });
                    }
                });
            }
        });
    });

    // Check leds
    if (!Array.isArray(config.leds) || config.leds.length !== 8) {
        errors.push('LEDs must be an array of 8 objects.');
    } else {
        config.leds.forEach((led, idx) => {
            if (typeof led !== 'object' || led === null) {
                errors.push(`LED ${idx + 1}: Must be an object.`);
            } else {
                ['r', 'g', 'b'].forEach(c => {
                    if (typeof led[c] !== 'number' || led[c] < 0 || led[c] > 255 || !Number.isInteger(led[c])) {
                        errors.push(`LED ${idx + 1}: ${c} must be an integer 0-255.`);
                    }
                });
            }
        });
    }

    return errors;
}

function loadConfigIntoForm(config) {
    elements.configName.value = config.name || '';

    buttonFields.forEach((field, idx) => {
        const btn = config[field.id];
        if (btn) {
            document.getElementById(`${field.id}-text`).value = btn.display_text || '';
            document.getElementById(`${field.id}-icon`).value = btn.display_icon || '';
            const keystrokes = btn.keystroke || [];
            const text = keystrokes.map(chord => chord.join(',')).join('\n');
            document.getElementById(`${field.id}-keystrokes`).value = text;
        }
    });

    const encoderIds = ['menu_encoder', 'encoder1', 'encoder2'];
    encoderIds.forEach(encId => {
        const enc = config[encId];
        if (enc) {
            document.getElementById(`${encId}-text`).value = enc.display_text || '';
            document.getElementById(`${encId}-icon`).value = enc.display_icon || '';
            const types = encId === 'menu_encoder' ? ['left', 'right'] : ['left', 'right', 'push'];
            types.forEach(type => {
                const keystrokes = enc[`keystroke_${type}`] || [];
                const text = keystrokes.map(chord => chord.join(',')).join('\n');
                document.getElementById(`${encId}-${type}-keystrokes`).value = text;
            });
        }
    });

    if (config.leds && Array.isArray(config.leds)) {
        config.leds.forEach((led, idx) => {
            if (led && typeof led === 'object') {
                const r = Math.max(0, Math.min(255, led.r || 0));
                const g = Math.max(0, Math.min(255, led.g || 0));
                const b = Math.max(0, Math.min(255, led.b || 0));
                const hex = `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`;
                document.getElementById(`led-${idx}`).value = hex;
            }
        });
    }
}

function loadJson() {
    const file = elements.loadJsonFile.files[0];
    if (!file) {
        alert('Please select a JSON file first.');
        return;
    }
    const reader = new FileReader();
    reader.onload = () => {
        try {
            const config = JSON.parse(reader.result);
            loadConfigIntoForm(config);
            updateOutput();
        } catch (e) {
            alert('Invalid JSON file: ' + e.message);
        }
    };
    reader.readAsText(file);
}

function updateOutput() {
    const config = buildConfig();
    const errors = validateConfig(config);
    if (errors.length > 0) {
        elements.validationErrors.innerHTML = '<ul>' + errors.map(e => `<li>${e}</li>`).join('') + '</ul>';
    } else {
        elements.validationErrors.innerHTML = '';
    }
    elements.outputJson.value = JSON.stringify(config, null, 4);
    updateDisplayPreview(config);
}

function setButtonIconPreview(buttonId, blob) {
    const preview = document.getElementById(`${buttonId}-icon-preview`);
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

function updateButtonIconFromFile(buttonId) {
    const input = document.getElementById(`${buttonId}-icon-file`);
    if (!(input instanceof HTMLInputElement) || !input.files?.[0]) {
        return;
    }
    const file = input.files[0];
    const reader = new FileReader();
    reader.onload = () => {
        const image = new Image();
        image.onload = () => {
            const bmpBlob = convertIconImage(image);
            if (!bmpBlob) {
                return;
            }
            const fileName = file.name.replace(/\.[^.]+$/, '.bmp');
            buttonIconBlobs.set(buttonId, { blob: bmpBlob, name: fileName });
            const iconNameInput = document.getElementById(`${buttonId}-icon`);
            if (iconNameInput instanceof HTMLInputElement) {
                iconNameInput.value = fileName;
            }
            const downloadBtn = document.getElementById(`${buttonId}-icon-download`);
            if (downloadBtn instanceof HTMLButtonElement) {
                downloadBtn.disabled = false;
            }
            setButtonIconPreview(buttonId, bmpBlob);
            updateOutput();
        };
        image.src = reader.result;
    };
    reader.readAsDataURL(file);
}

function downloadButtonIcon(buttonId) {
    const icon = buttonIconBlobs.get(buttonId);
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

function sanitizeFilename(name) {
    // Sanitize to 8.3 format: uppercase, alphanum/underscore, max 8 chars, .CFG
    let base = name.replace(/[^a-zA-Z0-9_]/g, '').toUpperCase();
    if (base.length === 0) base = 'CONFIG';
    base = base.substring(0, 8);
    return base + '.CFG';
}

function downloadJson() {
    const content = elements.outputJson.value;
    const blob = new Blob([content], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    const filename = sanitizeFilename(elements.configName.value.trim() || 'macro_config');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
}

function copyJson() {
    navigator.clipboard.writeText(elements.outputJson.value).then(() => {
        alert('JSON copied to clipboard');
    });
}

function render() {
    elements.buttonsGrid.innerHTML = buttonFields.map(createButtonCard).join('');
    elements.encodersGrid.innerHTML = encoderFields.map(createEncoderCard).join('');
    elements.ledsGrid.innerHTML = Array.from({ length: 8 }, (_, idx) => createLedCard(idx)).join('');
    elements.keycodeList.textContent = KEY_CODES.join(', ');
}

function loadExampleConfig() {
    const example = {
        name: 'example',
    };

    buttonFields.forEach((field, idx) => {
        const char = String.fromCharCode(65 + idx);
        example[field.id] = {
            display_text: char,
            display_icon: null,
            keystroke: [[char]],
        };
    });

    example.menu_encoder = {
        display_text: 'Vol',
        display_icon: null,
        keystroke_left: [['VolumeDown']],
        keystroke_right: [['VolumeUp']],
    };

    example.encoder1 = {
        display_text: 'Copy/Paste',
        display_icon: null,
        keystroke_left: [['LeftControl', 'C']],
        keystroke_right: [['LeftControl', 'V']],
        keystroke_push: [['LeftControl', 'V']],
    };

    example.encoder2 = { ...example.encoder1 };
    example.leds = [
        { r: 255, g: 0, b: 0 },
        { r: 0, g: 255, b: 0 },
        { r: 0, g: 0, b: 255 },
        { r: 255, g: 255, b: 0 },
        { r: 255, g: 0, b: 255 },
        { r: 0, g: 255, b: 255 },
        { r: 255, g: 255, b: 255 },
        { r: 128, g: 128, b: 128 },
    ];

    elements.configName.value = example.name;

    buttonFields.forEach(field => {
        document.getElementById(`${field.id}-text`).value = example[field.id].display_text;
        document.getElementById(`${field.id}-icon`).value = '';
        document.getElementById(`${field.id}-keystrokes`).value = example[field.id].keystroke
            .map(chord => chord.join(','))
            .join('\n');
    });

    encoderFields.forEach(field => {
        document.getElementById(`${field.id}-text`).value = example[field.id].display_text;
        document.getElementById(`${field.id}-icon`).value = '';
        field.types.forEach(type => {
            document.getElementById(`${field.id}-${type}-keystrokes`).value = example[field.id][`keystroke_${type}`]
                .map(chord => chord.join(','))
                .join('\n');
        });
    });

    example.leds.forEach((led, idx) => {
        const hex = `#${led.r.toString(16).padStart(2, '0')}${led.g.toString(16).padStart(2, '0')}${led.b.toString(16).padStart(2, '0')}`;
        document.getElementById(`led-${idx}`).value = hex;
    });

    updateOutput();
}

render();
loadExampleConfig();

elements.updateJsonBtn.addEventListener('click', updateOutput);
elements.copyJsonBtn.addEventListener('click', copyJson);
elements.downloadJsonBtn.addEventListener('click', downloadJson);
elements.convertIconBtn.addEventListener('click', updateIconFromFile);
elements.downloadIconBtn.addEventListener('click', downloadIcon);
elements.loadExampleBtn.addEventListener('click', loadExampleConfig);
elements.loadJsonBtn.addEventListener('click', loadJson);

document.addEventListener('change', event => {
    const target = event.target;
    if (!(target instanceof HTMLInputElement)) {
        return;
    }
    if (target.classList.contains('button-icon-file')) {
        const buttonId = target.id.replace(/-icon-file$/, '');
        updateButtonIconFromFile(buttonId);
    }
});

document.addEventListener('click', event => {
    const target = event.target;
    if (!(target instanceof HTMLButtonElement)) {
        return;
    }
    if (target.classList.contains('icon-download')) {
        const buttonId = target.id.replace(/-icon-download$/, '');
        downloadButtonIcon(buttonId);
    }
});

[...document.querySelectorAll('input, textarea')].forEach(el => {
    el.addEventListener('input', updateOutput);
});
elements.iconFile.addEventListener('change', updateIconFromFile);
