import React, { useState, useEffect } from 'react';
import { useTradingStore } from '../stores/tradingStore';
import { Button } from './ui/Button';
import { Input } from './ui/Input';
import { Select } from './ui/Select';
import { Card } from './ui/Card';
import { Badge } from './ui/Badge';
import { Alert, AlertCircle, Bell, BellOff, Trash2, Plus, Settings } from 'lucide-react';
import { toast } from 'react-hot-toast';

interface StockAlert {
  id: string;
  symbol: string;
  alertType: 'price_above' | 'price_below' | 'percentage_gain' | 'percentage_loss' | 'volume_spike' | 'news_mention' | 'earnings_announcement' | 'technical_breakout';
  priceTarget?: number;
  percentageChange?: number;
  isActive: boolean;
  createdAt: string;
  confidence?: number;
  reasoning?: string;
}

interface NewsAlert {
  id: string;
  symbol: string;
  keywords: string[];
  sources: string[];
  isActive: boolean;
  createdAt: string;
}

interface AlertInsight {
  symbol: string;
  alertType: string;
  confidence: number;
  reasoning: string;
  suggestedAction: string;
  riskLevel: 'low' | 'medium' | 'high';
}

export const AlertManager: React.FC = () => {
  const { apiClient } = useTradingStore();
  const [alerts, setAlerts] = useState<StockAlert[]>([]);
  const [newsAlerts, setNewsAlerts] = useState<NewsAlert[]>([]);
  const [insights, setInsights] = useState<AlertInsight[]>([]);
  const [loading, setLoading] = useState(false);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [showInsights, setShowInsights] = useState(false);
  
  // Form state
  const [formData, setFormData] = useState({
    symbol: '',
    alertType: 'price_above' as const,
    priceTarget: '',
    percentageChange: '',
    keywords: '',
    sources: '',
  });

  useEffect(() => {
    loadAlerts();
    loadInsights();
  }, []);

  const loadAlerts = async () => {
    try {
      setLoading(true);
      const response = await apiClient.get('/api/alerts');
      setAlerts(response.data.priceAlerts || []);
      setNewsAlerts(response.data.newsAlerts || []);
    } catch (error) {
      toast.error('Failed to load alerts');
      console.error('Error loading alerts:', error);
    } finally {
      setLoading(false);
    }
  };

  const loadInsights = async () => {
    try {
      const response = await apiClient.get('/api/copilot/alert-insights');
      setInsights(response.data.insights || []);
    } catch (error) {
      console.error('Error loading insights:', error);
    }
  };

  const createAlert = async () => {
    try {
      setLoading(true);
      
      if (formData.alertType === 'news_mention') {
        const newsAlert = {
          symbol: formData.symbol.toUpperCase(),
          keywords: formData.keywords.split(',').map(k => k.trim()).filter(Boolean),
          sources: formData.sources.split(',').map(s => s.trim()).filter(Boolean),
        };
        
        await apiClient.post('/api/alerts/news', newsAlert);
        toast.success('News alert created successfully');
      } else {
        const alert = {
          symbol: formData.symbol.toUpperCase(),
          alertType: formData.alertType,
          priceTarget: formData.priceTarget ? parseFloat(formData.priceTarget) : undefined,
          percentageChange: formData.percentageChange ? parseFloat(formData.percentageChange) : undefined,
        };
        
        await apiClient.post('/api/alerts/price', alert);
        toast.success('Price alert created successfully');
      }
      
      setShowCreateForm(false);
      setFormData({
        symbol: '',
        alertType: 'price_above',
        priceTarget: '',
        percentageChange: '',
        keywords: '',
        sources: '',
      });
      loadAlerts();
    } catch (error) {
      toast.error('Failed to create alert');
      console.error('Error creating alert:', error);
    } finally {
      setLoading(false);
    }
  };

  const toggleAlert = async (alertId: string, isActive: boolean) => {
    try {
      await apiClient.patch(`/api/alerts/${alertId}`, { isActive: !isActive });
      toast.success(`Alert ${isActive ? 'deactivated' : 'activated'}`);
      loadAlerts();
    } catch (error) {
      toast.error('Failed to update alert');
      console.error('Error updating alert:', error);
    }
  };

  const deleteAlert = async (alertId: string) => {
    if (!confirm('Are you sure you want to delete this alert?')) return;
    
    try {
      await apiClient.delete(`/api/alerts/${alertId}`);
      toast.success('Alert deleted successfully');
      loadAlerts();
    } catch (error) {
      toast.error('Failed to delete alert');
      console.error('Error deleting alert:', error);
    }
  };

  const createFromInsight = async (insight: AlertInsight) => {
    try {
      setLoading(true);
      
      const alert = {
        symbol: insight.symbol,
        alertType: insight.alertType,
        confidence: insight.confidence,
        reasoning: insight.reasoning,
      };
      
      await apiClient.post('/api/alerts/from-insight', alert);
      toast.success('Alert created from AI insight');
      loadAlerts();
      loadInsights();
    } catch (error) {
      toast.error('Failed to create alert from insight');
      console.error('Error creating alert from insight:', error);
    } finally {
      setLoading(false);
    }
  };

  const getAlertTypeLabel = (type: string) => {
    const labels: Record<string, string> = {
      price_above: 'Price Above',
      price_below: 'Price Below',
      percentage_gain: 'Percentage Gain',
      percentage_loss: 'Percentage Loss',
      volume_spike: 'Volume Spike',
      news_mention: 'News Mention',
      earnings_announcement: 'Earnings',
      technical_breakout: 'Technical Breakout',
    };
    return labels[type] || type;
  };

  const getRiskLevelColor = (level: string) => {
    switch (level) {
      case 'low': return 'bg-green-100 text-green-800';
      case 'medium': return 'bg-yellow-100 text-yellow-800';
      case 'high': return 'bg-red-100 text-red-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">Alert Manager</h2>
          <p className="text-gray-600">Monitor stocks and get notified about important events</p>
        </div>
        <div className="flex space-x-2">
          <Button
            variant="outline"
            onClick={() => setShowInsights(!showInsights)}
            className="flex items-center space-x-2"
          >
            <Settings className="w-4 h-4" />
            <span>AI Insights</span>
          </Button>
          <Button
            onClick={() => setShowCreateForm(!showCreateForm)}
            className="flex items-center space-x-2"
          >
            <Plus className="w-4 h-4" />
            <span>Create Alert</span>
          </Button>
        </div>
      </div>

      {/* AI Insights */}
      {showInsights && (
        <Card className="p-6">
          <h3 className="text-lg font-semibold mb-4 flex items-center space-x-2">
            <AlertCircle className="w-5 h-5 text-blue-600" />
            <span>AI-Powered Alert Suggestions</span>
          </h3>
          
          {insights.length === 0 ? (
            <div className="text-center py-8">
              <p className="text-gray-500">No AI insights available. Add stocks to your watchlist to get suggestions.</p>
            </div>
          ) : (
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {insights.map((insight, index) => (
                <div key={index} className="border rounded-lg p-4 space-y-3">
                  <div className="flex items-center justify-between">
                    <span className="font-semibold text-lg">{insight.symbol}</span>
                    <Badge className={getRiskLevelColor(insight.riskLevel)}>
                      {insight.riskLevel}
                    </Badge>
                  </div>
                  
                  <div>
                    <p className="text-sm font-medium text-gray-700">
                      {getAlertTypeLabel(insight.alertType)}
                    </p>
                    <p className="text-sm text-gray-600 mt-1">{insight.reasoning}</p>
                  </div>
                  
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-gray-500">
                      Confidence: {Math.round(insight.confidence * 100)}%
                    </span>
                    <Button
                      size="sm"
                      onClick={() => createFromInsight(insight)}
                      disabled={loading}
                    >
                      Create Alert
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </Card>
      )}

      {/* Create Alert Form */}
      {showCreateForm && (
        <Card className="p-6">
          <h3 className="text-lg font-semibold mb-4">Create New Alert</h3>
          
          <div className="grid gap-4 md:grid-cols-2">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Symbol
              </label>
              <Input
                value={formData.symbol}
                onChange={(e) => setFormData({ ...formData, symbol: e.target.value })}
                placeholder="AAPL"
                className="w-full"
              />
            </div>
            
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Alert Type
              </label>
              <Select
                value={formData.alertType}
                onValueChange={(value) => setFormData({ ...formData, alertType: value as any })}
              >
                <option value="price_above">Price Above</option>
                <option value="price_below">Price Below</option>
                <option value="percentage_gain">Percentage Gain</option>
                <option value="percentage_loss">Percentage Loss</option>
                <option value="volume_spike">Volume Spike</option>
                <option value="news_mention">News Mention</option>
                <option value="earnings_announcement">Earnings Announcement</option>
                <option value="technical_breakout">Technical Breakout</option>
              </Select>
            </div>
            
            {formData.alertType !== 'news_mention' && (
              <>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Price Target
                  </label>
                  <Input
                    type="number"
                    step="0.01"
                    value={formData.priceTarget}
                    onChange={(e) => setFormData({ ...formData, priceTarget: e.target.value })}
                    placeholder="150.00"
                    className="w-full"
                  />
                </div>
                
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Percentage Change
                  </label>
                  <Input
                    type="number"
                    step="0.1"
                    value={formData.percentageChange}
                    onChange={(e) => setFormData({ ...formData, percentageChange: e.target.value })}
                    placeholder="5.0"
                    className="w-full"
                  />
                </div>
              </>
            )}
            
            {formData.alertType === 'news_mention' && (
              <>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Keywords (comma-separated)
                  </label>
                  <Input
                    value={formData.keywords}
                    onChange={(e) => setFormData({ ...formData, keywords: e.target.value })}
                    placeholder="earnings, revenue, growth"
                    className="w-full"
                  />
                </div>
                
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Sources (comma-separated)
                  </label>
                  <Input
                    value={formData.sources}
                    onChange={(e) => setFormData({ ...formData, sources: e.target.value })}
                    placeholder="Reuters, Bloomberg, Yahoo Finance"
                    className="w-full"
                  />
                </div>
              </>
            )}
          </div>
          
          <div className="flex space-x-2 mt-6">
            <Button onClick={createAlert} disabled={loading}>
              {loading ? 'Creating...' : 'Create Alert'}
            </Button>
            <Button
              variant="outline"
              onClick={() => setShowCreateForm(false)}
              disabled={loading}
            >
              Cancel
            </Button>
          </div>
        </Card>
      )}

      {/* Active Alerts */}
      <div className="space-y-4">
        <h3 className="text-lg font-semibold">Active Alerts</h3>
        
        {loading ? (
          <div className="text-center py-8">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"></div>
            <p className="text-gray-500 mt-2">Loading alerts...</p>
          </div>
        ) : alerts.length === 0 && newsAlerts.length === 0 ? (
          <Card className="p-8 text-center">
            <Bell className="w-12 h-12 text-gray-400 mx-auto mb-4" />
            <h4 className="text-lg font-medium text-gray-900 mb-2">No alerts yet</h4>
            <p className="text-gray-500 mb-4">
              Create your first alert to start monitoring stocks and get notified about important events.
            </p>
            <Button onClick={() => setShowCreateForm(true)}>
              Create Your First Alert
            </Button>
          </Card>
        ) : (
          <div className="space-y-4">
            {/* Price Alerts */}
            {alerts.map((alert) => (
              <Card key={alert.id} className="p-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-4">
                    <div className="flex items-center space-x-2">
                      {alert.isActive ? (
                        <Bell className="w-5 h-5 text-green-600" />
                      ) : (
                        <BellOff className="w-5 h-5 text-gray-400" />
                      )}
                      <span className="font-semibold text-lg">{alert.symbol}</span>
                    </div>
                    
                    <Badge variant="outline">
                      {getAlertTypeLabel(alert.alertType)}
                    </Badge>
                    
                    {alert.priceTarget && (
                      <span className="text-sm text-gray-600">
                        Target: ${alert.priceTarget}
                      </span>
                    )}
                    
                    {alert.percentageChange && (
                      <span className="text-sm text-gray-600">
                        {alert.percentageChange}%
                      </span>
                    )}
                    
                    {alert.confidence && (
                      <span className="text-sm text-gray-500">
                        {Math.round(alert.confidence * 100)}% confidence
                      </span>
                    )}
                  </div>
                  
                  <div className="flex items-center space-x-2">
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => toggleAlert(alert.id, alert.isActive)}
                    >
                      {alert.isActive ? 'Deactivate' : 'Activate'}
                    </Button>
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => deleteAlert(alert.id)}
                      className="text-red-600 hover:text-red-700"
                    >
                      <Trash2 className="w-4 h-4" />
                    </Button>
                  </div>
                </div>
                
                {alert.reasoning && (
                  <p className="text-sm text-gray-600 mt-2">{alert.reasoning}</p>
                )}
              </Card>
            ))}
            
            {/* News Alerts */}
            {newsAlerts.map((alert) => (
              <Card key={alert.id} className="p-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-4">
                    <div className="flex items-center space-x-2">
                      {alert.isActive ? (
                        <Bell className="w-5 h-5 text-green-600" />
                      ) : (
                        <BellOff className="w-5 h-5 text-gray-400" />
                      )}
                      <span className="font-semibold text-lg">{alert.symbol}</span>
                    </div>
                    
                    <Badge variant="outline">News Alert</Badge>
                    
                    <div className="text-sm text-gray-600">
                      Keywords: {alert.keywords.join(', ')}
                    </div>
                    
                    <div className="text-sm text-gray-600">
                      Sources: {alert.sources.join(', ')}
                    </div>
                  </div>
                  
                  <div className="flex items-center space-x-2">
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => toggleAlert(alert.id, alert.isActive)}
                    >
                      {alert.isActive ? 'Deactivate' : 'Activate'}
                    </Button>
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => deleteAlert(alert.id)}
                      className="text-red-600 hover:text-red-700"
                    >
                      <Trash2 className="w-4 h-4" />
                    </Button>
                  </div>
                </div>
              </Card>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
