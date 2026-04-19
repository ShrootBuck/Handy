import React from "react";
import { useTranslation } from "react-i18next";
import { ModelCard } from "@/components/onboarding";
import { useModelStore } from "@/stores/modelStore";

export const ModelsSettings: React.FC = () => {
  const { t } = useTranslation();
  const { models, currentModel, loading, selectModel } = useModelStore();

  if (loading) {
    return (
      <div className="max-w-3xl w-full mx-auto">
        <div className="flex items-center justify-center py-16">
          <div className="w-8 h-8 border-2 border-logo-primary border-t-transparent rounded-full animate-spin" />
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-3xl w-full mx-auto space-y-4">
      <div className="mb-4">
        <h1 className="text-xl font-semibold mb-2">
          {t("settings.models.title")}
        </h1>
        <p className="text-sm text-text/60">
          {t("settings.models.description")}
        </p>
      </div>
      <div className="space-y-3">
        {models.map((model) => (
          <ModelCard
            key={model.id}
            model={model}
            status={model.id === currentModel ? "active" : "available"}
            onSelect={selectModel}
            showRecommended={false}
          />
        ))}
      </div>
    </div>
  );
};
