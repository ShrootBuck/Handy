import React from "react";
import { useTranslation } from "react-i18next";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { MistralTranscriptionSettings } from "../MistralTranscriptionSettings";
import { useModelStore } from "../../../stores/modelStore";

export const ModelSettingsCard: React.FC = () => {
  const { t } = useTranslation();
  const { currentModel, models } = useModelStore();

  const currentModelInfo = models.find((model) => model.id === currentModel);

  if (!currentModelInfo || currentModelInfo.engine_type !== "MistralApi") {
    return null;
  }

  return (
    <SettingsGroup
      title={t("settings.modelSettings.title", {
        model: currentModelInfo.name,
      })}
    >
      <MistralTranscriptionSettings />
    </SettingsGroup>
  );
};
