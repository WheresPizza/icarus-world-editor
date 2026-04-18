<script lang="ts">
  import type { ProspectInfo } from "../types";

  let { info, onSave }: { info: ProspectInfo; onSave: (info: ProspectInfo) => void } = $props();

  let editInfo = $state<ProspectInfo>(JSON.parse(JSON.stringify(info)));
  let dirty = $derived(JSON.stringify(editInfo) !== JSON.stringify(info));

  function handleSave() {
    onSave(editInfo);
  }

  function handleDiscard() {
    editInfo = JSON.parse(JSON.stringify(info));
  }
</script>

<div class="metadata-editor">
  <div class="editor-header">
    <h3>Prospect Metadata</h3>
    {#if dirty}
      <div class="dirty-actions">
        <span class="dirty-badge">Unsaved changes</span>
        <button class="btn btn-secondary" onclick={handleDiscard}>Discard</button>
        <button class="btn btn-primary" onclick={handleSave}>Save</button>
      </div>
    {/if}
  </div>

  <div class="form-grid">
    <div class="form-group">
      <label for="prospect-id">Prospect ID</label>
      <input id="prospect-id" type="text" bind:value={editInfo.ProspectID} />
    </div>

    <div class="form-group">
      <label for="prospect-key">Mission Type</label>
      <input id="prospect-key" type="text" bind:value={editInfo.ProspectDTKey} />
    </div>

    <div class="form-group">
      <label for="difficulty">Difficulty</label>
      <select id="difficulty" bind:value={editInfo.Difficulty}>
        <option value="Easy">Easy</option>
        <option value="Medium">Medium</option>
        <option value="Hard">Hard</option>
        <option value="Extreme">Extreme</option>
      </select>
    </div>

    <div class="form-group">
      <label for="state">State</label>
      <select id="state" bind:value={editInfo.ProspectState}>
        <option value="Active">Active</option>
        <option value="Complete">Complete</option>
        <option value="Failed">Failed</option>
        <option value="Abandoned">Abandoned</option>
      </select>
    </div>

    <div class="form-group">
      <label for="lobby-name">Lobby Name</label>
      <input id="lobby-name" type="text" bind:value={editInfo.LobbyName} />
    </div>

    <div class="form-group">
      <label for="elapsed">Elapsed Time (seconds)</label>
      <input id="elapsed" type="number" bind:value={editInfo.ElapsedTime} />
    </div>

    <div class="form-group">
      <label for="expire">Expire Time</label>
      <input id="expire" type="number" bind:value={editInfo.ExpireTime} />
    </div>

    <div class="form-group">
      <label for="drop-point">Drop Point</label>
      <input id="drop-point" type="number" bind:value={editInfo.SelectedDropPoint} />
    </div>

    <div class="form-group">
      <label for="cost">Cost</label>
      <input id="cost" type="number" bind:value={editInfo.Cost} />
    </div>

    <div class="form-group">
      <label for="reward">Reward</label>
      <input id="reward" type="number" bind:value={editInfo.Reward} />
    </div>

    <div class="form-group toggle-group">
      <label>
        <input type="checkbox" bind:checked={editInfo.Insurance} />
        Insurance
      </label>
    </div>

    <div class="form-group toggle-group">
      <label>
        <input type="checkbox" bind:checked={editInfo.NoRespawns} />
        No Respawns
      </label>
    </div>
  </div>

  <div class="section">
    <h4>Players ({editInfo.AssociatedMembers.length})</h4>
    <div class="players-list">
      {#each editInfo.AssociatedMembers as member, i}
        <div class="player-row">
          <span class="player-name">{member.CharacterName}</span>
          <span class="player-id">{member.UserID}</span>
          <span class="player-xp">XP: {member.Experience}</span>
          <span class="player-status" class:online={member.IsCurrentlyPlaying}>
            {member.IsCurrentlyPlaying ? "Online" : "Offline"}
          </span>
        </div>
      {/each}
    </div>
  </div>

  {#if editInfo.CustomSettings.length > 0}
    <div class="section">
      <h4>Custom Settings</h4>
      <div class="settings-list">
        {#each editInfo.CustomSettings as setting}
          <div class="setting-row">
            <span class="setting-name">{setting.SettingRowName}</span>
            <input
              class="setting-value"
              type="text"
              value={JSON.stringify(setting.SettingValue)}
              onchange={(e) => {
                try {
                  setting.SettingValue = JSON.parse((e.target as HTMLInputElement).value);
                } catch {}
              }}
            />
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .metadata-editor {
    padding: 20px;
  }

  .editor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  .editor-header h3 {
    font-size: 16px;
    color: var(--text-primary);
  }

  .dirty-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .dirty-badge {
    font-size: 12px;
    color: var(--accent-orange);
    padding: 4px 8px;
    background: rgba(230, 126, 34, 0.1);
    border-radius: var(--radius-sm);
  }

  .btn {
    padding: 6px 16px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 500;
  }

  .btn-primary {
    background: var(--accent-blue);
    color: white;
  }

  .btn-primary:hover {
    background: var(--accent-blue-hover);
  }

  .btn-secondary {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    border: 1px solid var(--border-color);
  }

  .btn-secondary:hover {
    background: var(--bg-hover);
  }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    margin-bottom: 24px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .form-group label {
    font-size: 12px;
    color: var(--text-secondary);
    font-weight: 500;
  }

  .toggle-group label {
    flex-direction: row;
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-size: 13px;
    color: var(--text-primary);
  }

  .section {
    margin-bottom: 20px;
  }

  .section h4 {
    font-size: 14px;
    color: var(--text-primary);
    margin-bottom: 8px;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--border-color);
  }

  .player-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px;
    border-radius: var(--radius-sm);
    background: var(--bg-tertiary);
    margin-bottom: 4px;
  }

  .player-name {
    font-weight: 500;
    color: var(--text-primary);
    min-width: 150px;
  }

  .player-id {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-muted);
  }

  .player-xp {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .player-status {
    font-size: 12px;
    color: var(--text-muted);
    margin-left: auto;
  }

  .player-status.online {
    color: var(--accent-green);
  }

  .setting-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 0;
  }

  .setting-name {
    font-size: 13px;
    color: var(--text-secondary);
    min-width: 200px;
  }

  .setting-value {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 12px;
  }

  .players-list, .settings-list {
    max-height: 200px;
    overflow-y: auto;
  }
</style>
