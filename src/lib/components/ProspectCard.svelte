<script lang="ts">
  import type { ProspectSummary } from "../types";
  import { formatBytes, formatElapsedTime } from "../helpers";

  let { prospect, onOpen }: { prospect: ProspectSummary; onOpen: (p: ProspectSummary) => void } = $props();

  const difficultyColors: Record<string, string> = {
    Easy: "var(--accent-green)",
    Medium: "var(--accent-orange)",
    Hard: "var(--accent-red)",
    Extreme: "#9b59b6",
  };

  function getDifficultyColor(d: string): string {
    return difficultyColors[d] || "var(--text-secondary)";
  }
</script>

<button class="card" onclick={() => onOpen(prospect)}>
  <div class="card-header">
    <h3 class="card-title">{prospect.prospect_info.ProspectID}</h3>
    <span
      class="badge"
      style="background: {getDifficultyColor(prospect.prospect_info.Difficulty)}"
    >
      {prospect.prospect_info.Difficulty}
    </span>
  </div>

  <div class="card-meta">
    <span class="meta-item">
      {prospect.prospect_info.ProspectDTKey}
    </span>
    <span class="meta-item state" class:active={prospect.prospect_info.ProspectState === "Active"}>
      {prospect.prospect_info.ProspectState}
    </span>
  </div>

  <div class="card-details">
    <div class="detail">
      <span class="detail-label">Players</span>
      <span class="detail-value">{prospect.prospect_info.AssociatedMembers.length}</span>
    </div>
    <div class="detail">
      <span class="detail-label">Time</span>
      <span class="detail-value">{formatElapsedTime(prospect.prospect_info.ElapsedTime)}</span>
    </div>
    <div class="detail">
      <span class="detail-label">Size</span>
      <span class="detail-value">{formatBytes(prospect.file_size)}</span>
    </div>
  </div>

  {#if prospect.prospect_info.AssociatedMembers.length > 0}
    <div class="card-players">
      {#each prospect.prospect_info.AssociatedMembers as member}
        <span class="player-tag" class:online={member.IsCurrentlyPlaying}>
          {member.CharacterName}
        </span>
      {/each}
    </div>
  {/if}
</button>

<style>
  .card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: 16px;
    text-align: left;
    width: 100%;
    transition: all 0.15s ease;
  }

  .card:hover {
    background: var(--bg-hover);
    border-color: var(--accent-blue);
    box-shadow: var(--shadow-md);
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .card-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .badge {
    font-size: 11px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 12px;
    color: white;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .card-meta {
    display: flex;
    gap: 8px;
    margin-bottom: 12px;
  }

  .meta-item {
    font-size: 12px;
    color: var(--text-muted);
  }

  .meta-item.state {
    color: var(--text-secondary);
  }

  .meta-item.state.active {
    color: var(--accent-green);
  }

  .card-details {
    display: flex;
    gap: 16px;
    margin-bottom: 12px;
  }

  .detail {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .detail-label {
    font-size: 11px;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .detail-value {
    font-size: 13px;
    color: var(--text-primary);
    font-weight: 500;
  }

  .card-players {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .player-tag {
    font-size: 11px;
    padding: 2px 8px;
    background: var(--bg-tertiary);
    border-radius: 4px;
    color: var(--text-secondary);
  }

  .player-tag.online {
    color: var(--accent-green);
    background: rgba(46, 204, 113, 0.1);
  }
</style>
