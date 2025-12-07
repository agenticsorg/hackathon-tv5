import { useAppStore, Settings } from '../stores/appStore';
import './SettingsModal.css';

export function SettingsModal() {
  const { closeSettings, settings, updateSettings } = useAppStore();

  const handleChange = <K extends keyof Settings>(key: K, value: Settings[K]) => {
    updateSettings({ [key]: value });
  };

  return (
    <div className="modal-overlay" onClick={closeSettings}>
      <div className="settings-modal" onClick={e => e.stopPropagation()}>
        <div className="settings-header">
          <h2>Settings</h2>
          <button className="close-btn" onClick={closeSettings}>×</button>
        </div>

        <div className="settings-content">
          {/* Appearance */}
          <div className="settings-section">
            <h3>Appearance</h3>

            <div className="setting-item">
              <div className="setting-info">
                <label>Theme</label>
                <p className="setting-desc">Choose your preferred color scheme</p>
              </div>
              <select
                value={settings.theme}
                onChange={e => handleChange('theme', e.target.value as Settings['theme'])}
              >
                <option value="light">Light</option>
                <option value="dark">Dark</option>
                <option value="system">System</option>
              </select>
            </div>

            <div className="setting-item">
              <div className="setting-info">
                <label>Font size</label>
                <p className="setting-desc">Base font size for the editor</p>
              </div>
              <div className="setting-control">
                <input
                  type="range"
                  min="12"
                  max="24"
                  value={settings.fontSize}
                  onChange={e => handleChange('fontSize', parseInt(e.target.value))}
                />
                <span className="setting-value">{settings.fontSize}px</span>
              </div>
            </div>

            <div className="setting-item">
              <div className="setting-info">
                <label>Line width</label>
                <p className="setting-desc">Maximum width of the editor content</p>
              </div>
              <div className="setting-control">
                <input
                  type="range"
                  min="400"
                  max="1200"
                  step="50"
                  value={settings.lineWidth}
                  onChange={e => handleChange('lineWidth', parseInt(e.target.value))}
                />
                <span className="setting-value">{settings.lineWidth}px</span>
              </div>
            </div>
          </div>

          {/* Editor */}
          <div className="settings-section">
            <h3>Editor</h3>

            <div className="setting-item">
              <div className="setting-info">
                <label>Show line numbers</label>
                <p className="setting-desc">Display line numbers in the editor</p>
              </div>
              <label className="toggle">
                <input
                  type="checkbox"
                  checked={settings.showLineNumbers}
                  onChange={e => handleChange('showLineNumbers', e.target.checked)}
                />
                <span className="toggle-slider" />
              </label>
            </div>

            <div className="setting-item">
              <div className="setting-info">
                <label>Spellcheck</label>
                <p className="setting-desc">Enable spellchecking in the editor</p>
              </div>
              <label className="toggle">
                <input
                  type="checkbox"
                  checked={settings.spellcheck}
                  onChange={e => handleChange('spellcheck', e.target.checked)}
                />
                <span className="toggle-slider" />
              </label>
            </div>

            <div className="setting-item">
              <div className="setting-info">
                <label>Vim mode</label>
                <p className="setting-desc">Enable Vim keybindings</p>
              </div>
              <label className="toggle">
                <input
                  type="checkbox"
                  checked={settings.vimMode}
                  onChange={e => handleChange('vimMode', e.target.checked)}
                />
                <span className="toggle-slider" />
              </label>
            </div>
          </div>

          {/* Files */}
          <div className="settings-section">
            <h3>Files</h3>

            <div className="setting-item">
              <div className="setting-info">
                <label>Auto save</label>
                <p className="setting-desc">Automatically save changes</p>
              </div>
              <label className="toggle">
                <input
                  type="checkbox"
                  checked={settings.autoSave}
                  onChange={e => handleChange('autoSave', e.target.checked)}
                />
                <span className="toggle-slider" />
              </label>
            </div>

            {settings.autoSave && (
              <div className="setting-item">
                <div className="setting-info">
                  <label>Auto save interval</label>
                  <p className="setting-desc">Time to wait before saving (ms)</p>
                </div>
                <div className="setting-control">
                  <input
                    type="range"
                    min="1000"
                    max="30000"
                    step="1000"
                    value={settings.autoSaveInterval}
                    onChange={e => handleChange('autoSaveInterval', parseInt(e.target.value))}
                  />
                  <span className="setting-value">{settings.autoSaveInterval / 1000}s</span>
                </div>
              </div>
            )}
          </div>
        </div>

        <div className="settings-footer">
          <button className="mod-cta" onClick={closeSettings}>Done</button>
        </div>
      </div>
    </div>
  );
}
