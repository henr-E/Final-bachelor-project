import { QuantityWithUnits } from '@/store/sensor';

export function getFirstQuantity(
    q: Record<string, QuantityWithUnits>
): QuantityWithUnits | undefined {
    return Object.values(q).length === 0 ? undefined : Object.values(q)[0];
}
