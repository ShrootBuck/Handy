import React from "react";
import { SettingContainer } from "../ui/SettingContainer";
import { Input } from "../ui/Input";
import { useSettings } from "../../hooks/useSettings";

export const MistralTranscriptionSettings: React.FC = () => {
  const { getSetting, updateSetting, isUpdating } = useSettings();

  const apiKey = getSetting("mistral_transcription_api_key") || "";

  return (
    <SettingContainer
      title="Mistral API key"
      description="Handy sends recordings to Mistral's transcription API using your key."
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
        placeholder="Paste your Mistral API key"
      />
    </SettingContainer>
  );
};
