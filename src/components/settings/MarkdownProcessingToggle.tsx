import React from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";

interface MarkdownProcessingToggleProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const MarkdownProcessingToggle: React.FC<MarkdownProcessingToggleProps> =
  React.memo(({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();

    const enabled = getSetting("markdown_processing_enabled") || false;

    return (
      <ToggleSwitch
        checked={enabled}
        onChange={(enabled) =>
          updateSetting("markdown_processing_enabled", enabled)
        }
        isUpdating={isUpdating("markdown_processing_enabled")}
        label={t("settings.debug.markdownProcessing.label")}
        description={t("settings.debug.markdownProcessing.description")}
        descriptionMode={descriptionMode}
        grouped={grouped}
      />
    );
  });
