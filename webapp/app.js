const KEY_CODES = [
    "NoEventIndicated", "ErrorRollOver", "POSTFail", "ErrorUndefine", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "Keyboard1", "Keyboard2", "Keyboard3", "Keyboard4", "Keyboard5", "Keyboard6", "Keyboard7", "Keyboard8", "Keyboard9", "Keyboard0", "ReturnEnter", "Escape", "DeleteBackspace", "Tab", "Space", "Minus", "Equal", "LeftBrace", "RightBrace", "Backslash", "NonUSHash", "Semicolon", "Apostrophe", "Grave", "Comma", "Dot", "ForwardSlash", "CapsLock", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12", "PrintScreen", "ScrollLock", "Pause", "Insert", "Home", "PageUp", "DeleteForward", "End", "PageDown", "RightArrow", "LeftArrow", "DownArrow", "UpArrow", "KeypadNumLockAndClear", "KeypadDivide", "KeypadMultiply", "KeypadSubtract", "KeypadAdd", "KeypadEnter", "Keypad1", "Keypad2", "Keypad3", "Keypad4", "Keypad5", "Keypad6", "Keypad7", "Keypad8", "Keypad9", "Keypad0", "KeypadDot", "NonUSBackslash", "Application", "Power", "KeypadEqual", "F13", "F14", "F15", "F16", "F17", "F18", "F19", "F20", "F21", "F22", "F23", "F24", "Execute", "Help", "Menu", "Select", "Stop", "Again", "Undo", "Cut", "Copy", "Paste", "Find", "Mute", "VolumeUp", "VolumeDown", "LockingCapsLock", "LockingNumLock", "LockingScrollLock", "KeypadComma", "KeypadEqualSign", "Kanji1", "Kanji2", "Kanji3", "Kanji4", "Kanji5", "Kanji6", "Kanji7", "Kanji8", "Kanji9", "LANG1", "LANG2", "LANG3", "LANG4", "LANG5", "LANG6", "LANG7", "LANG8", "LANG9", "AlternateErase", "SysReqAttention", "Cancel", "Clear", "Prior", "Return", "Separator", "Out", "Oper", "ClearAgain", "CrSelProps", "ExSel", "LeftControl", "LeftShift", "LeftAlt", "LeftGUI", "RightControl", "RightShift", "RightAlt", "RightGUI"
];

const buttonFields = Array.from({ length: 10 }, (_, idx) => ({ id: `button${idx}`, title: `Button ${idx}` }));
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
};

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

function updateOutput() {
    const config = buildConfig();
    elements.outputJson.value = JSON.stringify(config, null, 4);
}

function downloadJson() {
    const content = elements.outputJson.value;
    const blob = new Blob([content], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${elements.configName.value.trim() || 'macro_config'}.json`;
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
elements.loadExampleBtn.addEventListener('click', loadExampleConfig);

[...document.querySelectorAll('input, textarea')].forEach(el => {
    el.addEventListener('input', updateOutput);
});
