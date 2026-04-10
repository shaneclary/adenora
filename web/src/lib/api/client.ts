import { get } from 'svelte/store';
import { authToken } from '../stores/auth';

const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:8080';

async function request(path: string, options: RequestInit = {}): Promise<any> {
  const token = get(authToken);
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string> || {}),
  };
  if (token) headers['Authorization'] = `Bearer ${token}`;

  const res = await fetch(`${API_BASE}${path}`, { ...options, headers });
  const data = await res.json();
  if (!res.ok) throw { status: res.status, ...data };
  return data;
}

export const api = {
  // Auth
  register: (body: any) => request('/api/v1/auth/register', { method: 'POST', body: JSON.stringify(body) }),
  login: (body: any) => request('/api/v1/auth/login', { method: 'POST', body: JSON.stringify(body) }),

  // Markets
  listMarkets: (params?: string) => request(`/api/v1/markets${params ? '?' + params : ''}`),
  getMarket: (id: string) => request(`/api/v1/markets/${id}`),
  getOrderbook: (id: string) => request(`/api/v1/markets/${id}/book`),
  proposeMarket: (body: any) => request('/api/v1/markets/propose', { method: 'POST', body: JSON.stringify(body) }),

  // Trading
  placeOrder: (body: any) => request('/api/v1/orders', { method: 'POST', body: JSON.stringify(body) }),
  cancelOrder: (id: string) => request(`/api/v1/orders/${id}/cancel`, { method: 'POST' }),
  getPositions: () => request('/api/v1/positions'),

  // Lottery
  listLotteries: () => request('/api/v1/lottery'),
  buyTicket: (id: string, numbers: number[]) => request(`/api/v1/lottery/${id}/tickets`, { method: 'POST', body: JSON.stringify({ numbers }) }),
  listDraws: (id: string) => request(`/api/v1/lottery/${id}/draws`),

  // Gaming
  listGames: () => request('/api/v1/games'),
  listTournaments: () => request('/api/v1/tournaments'),
  getLeaderboard: () => request('/api/v1/leaderboard'),

  // Bots
  listBots: () => request('/api/v1/bots'),
  registerBot: (body: any) => request('/api/v1/bots/register', { method: 'POST', body: JSON.stringify(body) }),
  getBot: (id: string) => request(`/api/v1/bots/${id}`),
  botLeaderboard: () => request('/api/v1/bots/leaderboard'),
  botTournaments: () => request('/api/v1/bots/tournaments'),

  // Charity
  listProjects: () => request('/api/v1/charity/projects'),
  getLedger: () => request('/api/v1/charity/ledger'),
  fundingSummary: () => request('/api/v1/charity/summary'),

  // Cause Campaigns
  listCampaigns: () => request('/api/v1/campaigns'),
  getCampaign: (slug: string) => request(`/api/v1/campaigns/${slug}`),
  getCampaignDonors: (slug: string) => request(`/api/v1/campaigns/${slug}/donors`),
  donate: (body: any) => request('/api/v1/campaigns/donate', { method: 'POST', body: JSON.stringify(body) }),

  // Human-friendly predict
  predict: (body: { market_id: string; side: string; amount_eur: number; public?: boolean; comment?: string }) =>
    request('/api/v1/predict', { method: 'POST', body: JSON.stringify(body) }),
  getMarketCard: (id: string) => request(`/api/v1/markets/${id}/card`),
  listMarketCards: () => request('/api/v1/markets/cards'),
  getPredictions: () => request('/api/v1/predictions'),
  getImpactFeed: () => request('/api/v1/impact/feed'),
  getMarketDebate: (id: string) => request(`/api/v1/markets/${id}/debate`),

  // Community Votes
  listVoteCampaigns: () => request('/api/v1/votes'),
  getVoteCampaign: (id: string) => request(`/api/v1/votes/${id}`),
  createVoteCampaign: (body: any) => request('/api/v1/votes', { method: 'POST', body: JSON.stringify(body) }),
  castVote: (campaignId: string, body: any) => request(`/api/v1/votes/${campaignId}/vote`, { method: 'POST', body: JSON.stringify(body) }),

  // Generic helpers used in some pages
  get: (path: string) => request(`/api/v1${path}`),
  post: (path: string, body: any) => request(`/api/v1${path}`, { method: 'POST', body: JSON.stringify(body) }),

  // Wallet
  getWallet: () => request('/api/v1/wallet'),
  deposit: (body: any) => request('/api/v1/wallet/deposit', { method: 'POST', body: JSON.stringify(body) }),
  withdraw: (body: any) => request('/api/v1/wallet/withdraw', { method: 'POST', body: JSON.stringify(body) }),
};
