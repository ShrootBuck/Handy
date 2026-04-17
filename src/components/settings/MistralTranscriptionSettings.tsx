import React from "react";
import { useTranslation } from "react-i18next";
import { SettingContainer } from "../ui/SettingContainer";
import { Input } from "../ui/Input";
import { useSettings } from "../../hooks/useSettings";

export const MistralTranscriptionSettings: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();

  const baseUrl = getSetting("mistral_transcription_base_url") || "";
  const apiKey = getSetting("mistral_transcription_api_key") || "";
  const model = getSetting("mistral_transcription_model") || "";

  return (
    <>
      <SettingContainer
        title={t("settings.modelSettings.mistral.baseUrl.title")}
        description={t("settings.modelSettings.mistral.baseUrl.description")}
        descriptionMode="inline"
        grouped={true}
        layout="stacked"
      >
        <Input
          type="text"
          value={baseUrl}
          onChange={(event) =>
            void updateSetting(
              "mistral_transcription_base_url",
              event.target.value,
            )
          }
          disabled={isUpdating("mistral_transcription_base_url")}
          className="w-full"
        />
      </SettingContainer>
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
      <SettingContainer
        title={t("settings.modelSettings.mistral.model.title")}
        description={t("settings.modelSettings.mistral.model.description")}
        descriptionMode="inline"
        grouped={true}
        layout="stacked"
      >
        <Input
          type="text"
          value={model}
          onChange={(event) =>
            void updateSetting(
              "mistral_transcription_model",
              event.target.value,
            )
          }
          disabled={isUpdating("mistral_transcription_model")}
          className="w-full"
        />
      </SettingContainer>
    </>
  );
};
