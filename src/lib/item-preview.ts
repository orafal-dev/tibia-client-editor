const ITEM_PREVIEW_BASE_URL = "https://item-images.ots.me/latest_otbr_anim";

export const getItemPreviewUrl = (itemId: string): string | null => {
  const trimmed = itemId.trim();
  if (!/^\d+$/.test(trimmed)) {
    return null;
  }

  return `${ITEM_PREVIEW_BASE_URL}/${trimmed}.gif`;
};
