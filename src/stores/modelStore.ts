import { create } from "zustand";
import { subscribeWithSelector } from "zustand/middleware";
import { commands, type ModelInfo } from "@/bindings";

interface ModelsStore {
  models: ModelInfo[];
  currentModel: string;
  loading: boolean;
  error: string | null;
  hasAnyModels: boolean;
  isFirstRun: boolean;
  initialized: boolean;
  initialize: () => Promise<void>;
  loadModels: () => Promise<void>;
  loadCurrentModel: () => Promise<void>;
  checkFirstRun: () => Promise<boolean>;
  selectModel: (modelId: string) => Promise<boolean>;
  getModelInfo: (modelId: string) => ModelInfo | undefined;
}

export const useModelStore = create<ModelsStore>()(
  subscribeWithSelector((set, get) => ({
    models: [],
    currentModel: "",
    loading: true,
    error: null,
    hasAnyModels: true,
    isFirstRun: false,
    initialized: false,

    loadModels: async () => {
      const result = await commands.getAvailableModels();
      if (result.status === "ok") {
        set({ models: result.data, error: null, loading: false });
      } else {
        set({ error: `Failed to load models: ${result.error}`, loading: false });
      }
    },

    loadCurrentModel: async () => {
      const result = await commands.getCurrentModel();
      if (result.status === "ok") {
        set({ currentModel: result.data });
      }
    },

    checkFirstRun: async () => {
      set({ hasAnyModels: true, isFirstRun: false });
      return false;
    },

    selectModel: async (modelId: string) => {
      const result = await commands.setActiveModel(modelId);
      if (result.status === "ok") {
        set({ currentModel: modelId, error: null });
        return true;
      }
      set({ error: `Failed to switch to model: ${result.error}` });
      return false;
    },

    getModelInfo: (modelId: string) => {
      return get().models.find((model) => model.id === modelId);
    },

    initialize: async () => {
      if (get().initialized) return;
      await Promise.all([get().loadModels(), get().loadCurrentModel()]);
      set({ initialized: true });
    },
  })),
);
