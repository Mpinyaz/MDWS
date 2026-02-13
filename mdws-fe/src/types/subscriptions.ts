import type { WsEvent } from './event'

export type AssetClass = 'forex' | 'equity' | 'crypto'

export interface SubscribePayload {
  assetClass: AssetClass
  tickers: Array<string>
}

export type UnsubscribePayload = SubscribePayload

export type UnsubscribeEvent = WsEvent<UnsubscribePayload>
