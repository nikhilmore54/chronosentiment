/**
 * Core pipeline keeps all price math in high-precision **100th of paise** (`PRICE_SCALE=10000`).
 * API responses are normalized to **real Rupees** at the boundary via PriceDto.
 * UI is strictly the display layer for real Rupees. No scaling operations are allowed.
 */

/**
 * Formats a real Rupee value for Indian standard display.
 * This is the ONLY allowed path for monetary display in the frontend.
 */
export function formatInr(amount: number, decimals = 2): string {
  if (amount === undefined || amount === null || !Number.isFinite(amount)) {
    return "₹0.00";
  }
  return `₹${amount.toLocaleString('en-IN', {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  })}`;
}

/** 
 * Simple precision-safe currency formatting utility.
 */
export function formatPrice(value: number): string {
    return value.toFixed(2);
}
