import React from "react";
import { useTranslation } from "react-i18next";
import { SettingContainer } from "../ui/SettingContainer";
import { Input } from "../ui/Input";
import { useSettings } from "../../hooks/useSettings";

export const MistralTranscriptionSettings: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();

  const apiKey = getSetting("mistral_transcription_api_key") || "";

  return (
    <SettingContainer
      title={t("settings.modelSettings.mistral.apiKey.title")}
      description={t("settings.modelSettings.mistral.apiKey.description")}
      descriptionMode="inline"
      grouped={true}
      layout="stacked"
    >
      <Input
        type="password"
        value={apiKey}
        onChange={(event) =>
          void updateSetting(
            "mistral_transcription_api_key",
            event.target.value,
          )
        }
        disabled={isUpdating("mistral_transcription_api_key")}
        className="w-full"
      />
    </SettingContainer>
  );
};
