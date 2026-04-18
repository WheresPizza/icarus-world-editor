import { writable } from "svelte/store";
import type {
  ProspectSummary,
  ProspectOverview,
  AppViewState,
  Property,
} from "../types";

export const prospects = writable<ProspectSummary[]>([]);
export const currentOverview = writable<ProspectOverview | null>(null);
export const currentComponentProps = writable<Property[] | null>(null);
export const loading = writable(false);
export const error = writable<string | null>(null);

export const viewState = writable<AppViewState>({
  mode: "library",
  selectedProspectId: null,
  selectedProspectPath: null,
  selectedComponentIndex: null,
});
