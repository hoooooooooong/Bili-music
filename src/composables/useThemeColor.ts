import { computed, watch, type ComputedRef, type Ref } from "vue";
import type { GlobalThemeOverrides } from "naive-ui";
import { deriveThemeColors } from "@/utils/colorUtils";

export function useThemeColor(accentColor: Ref<string> | ComputedRef<string>) {
  const themeOverrides = computed<GlobalThemeOverrides>(() => {
    const color = accentColor.value;
    const { hover, pressed } = deriveThemeColors(color);
    return {
      common: {
        primaryColor: color,
        primaryColorHover: hover,
        primaryColorPressed: pressed,
      },
      Switch: {
        railColorActive: color,
        loadingColor: color,
      },
    };
  });

  watch(
    accentColor,
    (color) => {
      const { hover, light } = deriveThemeColors(color);
      const root = document.documentElement;
      root.style.setProperty("--accent-color", color);
      root.style.setProperty("--accent-hover", hover);
      root.style.setProperty("--accent-light", light);
    },
    { immediate: true }
  );

  return { themeOverrides };
}
