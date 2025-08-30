import asyncio
import json
import logging
from datetime import datetime, timedelta
from typing import List, Dict, Optional, Any
from dataclasses import dataclass, asdict
from enum import Enum
import aiohttp
import openai
from pydantic import BaseModel, Field

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class AlertType(str, Enum):
    PRICE_ABOVE = "price_above"
    PRICE_BELOW = "price_below"
    PERCENTAGE_GAIN = "percentage_gain"
    PERCENTAGE_LOSS = "percentage_loss"
    VOLUME_SPIKE = "volume_spike"
    NEWS_MENTION = "news_mention"
    EARNINGS_ANNOUNCEMENT = "earnings_announcement"
    TECHNICAL_BREAKOUT = "technical_breakout"

class NotificationChannel(str, Enum):
    IN_APP = "in_app"
    EMAIL = "email"
    PUSH = "push"
    SMS = "sms"
    WEBSOCKET = "websocket"

@dataclass
class AlertRequest:
    symbol: str
    alert_type: AlertType
    price_target: Optional[float] = None
    percentage_change: Optional[float] = None
    notification_channels: List[NotificationChannel] = None
    keywords: List[str] = None
    sources: List[str] = None
    confidence: float = 0.8
    reasoning: str = ""

@dataclass
class NewsAlertRequest:
    symbol: str
    keywords: List[str]
    sources: List[str]
    notification_channels: List[NotificationChannel]
    confidence: float = 0.8
    reasoning: str = ""

@dataclass
class AlertInsight:
    symbol: str
    alert_type: AlertType
    confidence: float
    reasoning: str
    suggested_action: str
    market_context: str
    risk_level: str
    sources: List[str]

class AlertManager:
    def __init__(self, api_base_url: str = "http://localhost:8000"):
        self.api_base_url = api_base_url
        self.session = None

    async def __aenter__(self):
        self.session = aiohttp.ClientSession()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()

    async def create_price_alert(self, user_id: str, alert_request: AlertRequest) -> Dict[str, Any]:
        """Create a price alert for a user"""
        url = f"{self.api_base_url}/api/alerts/price"
        payload = {
            "user_id": user_id,
            "symbol": alert_request.symbol,
            "alert_type": alert_request.alert_type.value,
            "price_target": alert_request.price_target,
            "percentage_change": alert_request.percentage_change,
            "notification_channels": [ch.value for ch in alert_request.notification_channels or []]
        }
        
        async with self.session.post(url, json=payload) as response:
            return await response.json()

    async def create_news_alert(self, user_id: str, news_request: NewsAlertRequest) -> Dict[str, Any]:
        """Create a news alert for a user"""
        url = f"{self.api_base_url}/api/alerts/news"
        payload = {
            "user_id": user_id,
            "symbol": news_request.symbol,
            "keywords": news_request.keywords,
            "sources": news_request.sources,
            "notification_channels": [ch.value for ch in news_request.notification_channels]
        }
        
        async with self.session.post(url, json=payload) as response:
            return await response.json()

    async def get_user_alerts(self, user_id: str) -> List[Dict[str, Any]]:
        """Get all alerts for a user"""
        url = f"{self.api_base_url}/api/alerts/user/{user_id}"
        async with self.session.get(url) as response:
            return await response.json()

    async def delete_alert(self, alert_id: str) -> Dict[str, Any]:
        """Delete an alert"""
        url = f"{self.api_base_url}/api/alerts/{alert_id}"
        async with self.session.delete(url) as response:
            return await response.json()

class AlertAnalyzer:
    def __init__(self, openai_api_key: str):
        self.client = openai.AsyncOpenAI(api_key=openai_api_key)
        
    async def analyze_stock_for_alerts(self, symbol: str, current_price: float, 
                                     market_data: Dict[str, Any], user_interests: List[str]) -> List[AlertInsight]:
        """Analyze a stock and suggest relevant alerts based on current conditions and user interests"""
        
        prompt = f"""
        Analyze the stock {symbol} (current price: ${current_price:.2f}) and suggest intelligent alerts for a user interested in: {', '.join(user_interests)}.
        
        Market Data:
        - Current Price: ${current_price:.2f}
        - 52-week high: ${market_data.get('high_52w', 0):.2f}
        - 52-week low: ${market_data.get('low_52w', 0):.2f}
        - Volume: {market_data.get('volume', 0):,}
        - Average Volume: {market_data.get('avg_volume', 0):,}
        - P/E Ratio: {market_data.get('pe_ratio', 0):.2f}
        - Market Cap: ${market_data.get('market_cap', 0):,.0f}
        
        Consider:
        1. Technical levels (support/resistance)
        2. Volume patterns
        3. Earnings announcements
        4. News sentiment
        5. User's specific interests
        
        Suggest 2-4 relevant alerts with:
        - Alert type (price_above, price_below, percentage_gain, percentage_loss, volume_spike, news_mention, earnings_announcement, technical_breakout)
        - Target values
        - Confidence level (0.0-1.0)
        - Reasoning
        - Suggested action
        - Risk level (low, medium, high)
        
        Return as JSON array of alert suggestions.
        """
        
        try:
            response = await self.client.chat.completions.create(
                model="gpt-4",
                messages=[{"role": "user", "content": prompt}],
                temperature=0.3,
                max_tokens=1000
            )
            
            content = response.choices[0].message.content
            alert_suggestions = json.loads(content)
            
            insights = []
            for suggestion in alert_suggestions:
                insight = AlertInsight(
                    symbol=symbol,
                    alert_type=AlertType(suggestion['alert_type']),
                    confidence=suggestion['confidence'],
                    reasoning=suggestion['reasoning'],
                    suggested_action=suggestion['suggested_action'],
                    market_context=suggestion.get('market_context', ''),
                    risk_level=suggestion['risk_level'],
                    sources=suggestion.get('sources', [])
                )
                insights.append(insight)
            
            return insights
            
        except Exception as e:
            logger.error(f"Error analyzing stock {symbol}: {e}")
            return []

    async def suggest_news_alerts(self, symbol: str, user_interests: List[str], 
                                recent_news: List[Dict[str, Any]]) -> List[NewsAlertRequest]:
        """Suggest news alert configurations based on user interests and recent news patterns"""
        
        prompt = f"""
        Based on the user's interests in {', '.join(user_interests)} and recent news about {symbol}, 
        suggest intelligent news alert configurations.
        
        Recent News Patterns:
        {json.dumps(recent_news[:5], indent=2)}
        
        Suggest 2-3 news alert configurations with:
        - Relevant keywords
        - Preferred news sources
        - Reasoning for the configuration
        
        Return as JSON array of news alert suggestions.
        """
        
        try:
            response = await self.client.chat.completions.create(
                model="gpt-4",
                messages=[{"role": "user", "content": prompt}],
                temperature=0.3,
                max_tokens=800
            )
            
            content = response.choices[0].message.content
            suggestions = json.loads(content)
            
            news_requests = []
            for suggestion in suggestions:
                request = NewsAlertRequest(
                    symbol=symbol,
                    keywords=suggestion['keywords'],
                    sources=suggestion['sources'],
                    notification_channels=[NotificationChannel.IN_APP, NotificationChannel.PUSH],
                    confidence=suggestion.get('confidence', 0.8),
                    reasoning=suggestion['reasoning']
                )
                news_requests.append(request)
            
            return news_requests
            
        except Exception as e:
            logger.error(f"Error suggesting news alerts for {symbol}: {e}")
            return []

class CopilotAlertService:
    def __init__(self, openai_api_key: str, api_base_url: str = "http://localhost:8000"):
        self.analyzer = AlertAnalyzer(openai_api_key)
        self.api_base_url = api_base_url

    async def setup_intelligent_alerts(self, user_id: str, symbols: List[str], 
                                     user_interests: List[str], market_data: Dict[str, Dict[str, Any]]) -> Dict[str, Any]:
        """Set up intelligent alerts for multiple symbols based on user interests and market conditions"""
        
        results = {
            "created_alerts": [],
            "suggestions": [],
            "errors": []
        }
        
        async with AlertManager(self.api_base_url) as alert_manager:
            for symbol in symbols:
                try:
                    if symbol not in market_data:
                        results["errors"].append(f"No market data for {symbol}")
                        continue
                    
                    current_price = market_data[symbol].get('price', 0)
                    if current_price == 0:
                        results["errors"].append(f"Invalid price for {symbol}")
                        continue
                    
                    # Analyze stock for alert suggestions
                    insights = await self.analyzer.analyze_stock_for_alerts(
                        symbol, current_price, market_data[symbol], user_interests
                    )
                    
                    # Create alerts for high-confidence insights
                    for insight in insights:
                        if insight.confidence >= 0.7:
                            try:
                                alert_request = AlertRequest(
                                    symbol=insight.symbol,
                                    alert_type=insight.alert_type,
                                    price_target=self._get_price_target(insight, current_price),
                                    percentage_change=self._get_percentage_change(insight),
                                    notification_channels=[NotificationChannel.IN_APP, NotificationChannel.PUSH],
                                    confidence=insight.confidence,
                                    reasoning=insight.reasoning
                                )
                                
                                result = await alert_manager.create_price_alert(user_id, alert_request)
                                if result.get('success'):
                                    results["created_alerts"].append({
                                        "symbol": symbol,
                                        "alert_type": insight.alert_type.value,
                                        "confidence": insight.confidence,
                                        "reasoning": insight.reasoning
                                    })
                                else:
                                    results["errors"].append(f"Failed to create alert for {symbol}: {result.get('message', 'Unknown error')}")
                                    
                            except Exception as e:
                                results["errors"].append(f"Error creating alert for {symbol}: {e}")
                        else:
                            results["suggestions"].append({
                                "symbol": symbol,
                                "alert_type": insight.alert_type.value,
                                "confidence": insight.confidence,
                                "reasoning": insight.reasoning,
                                "suggested_action": insight.suggested_action
                            })
                
                except Exception as e:
                    results["errors"].append(f"Error processing {symbol}: {e}")
        
        return results

    async def setup_news_monitoring(self, user_id: str, symbols: List[str], 
                                  user_interests: List[str]) -> Dict[str, Any]:
        """Set up news monitoring for symbols based on user interests"""
        
        results = {
            "created_alerts": [],
            "errors": []
        }
        
        async with AlertManager(self.api_base_url) as alert_manager:
            for symbol in symbols:
                try:
                    # Get recent news for the symbol to understand patterns
                    recent_news = await self._get_recent_news(symbol)
                    
                    # Suggest news alert configurations
                    news_requests = await self.analyzer.suggest_news_alerts(
                        symbol, user_interests, recent_news
                    )
                    
                    # Create news alerts
                    for request in news_requests:
                        try:
                            result = await alert_manager.create_news_alert(user_id, request)
                            if result.get('success'):
                                results["created_alerts"].append({
                                    "symbol": symbol,
                                    "keywords": request.keywords,
                                    "sources": request.sources,
                                    "confidence": request.confidence
                                })
                            else:
                                results["errors"].append(f"Failed to create news alert for {symbol}: {result.get('message', 'Unknown error')}")
                                
                        except Exception as e:
                            results["errors"].append(f"Error creating news alert for {symbol}: {e}")
                
                except Exception as e:
                    results["errors"].append(f"Error processing news monitoring for {symbol}: {e}")
        
        return results

    async def get_alert_summary(self, user_id: str) -> Dict[str, Any]:
        """Get a summary of user's current alerts and recent activity"""
        
        async with AlertManager(self.api_base_url) as alert_manager:
            try:
                alerts = await alert_manager.get_user_alerts(user_id)
                
                summary = {
                    "total_alerts": len(alerts),
                    "active_alerts": len([a for a in alerts if a.get('is_active', True)]),
                    "alert_types": {},
                    "symbols": set(),
                    "recent_activity": []
                }
                
                for alert in alerts:
                    alert_type = alert.get('alert_type', 'unknown')
                    summary["alert_types"][alert_type] = summary["alert_types"].get(alert_type, 0) + 1
                    summary["symbols"].add(alert.get('symbol', ''))
                
                summary["symbols"] = list(summary["symbols"])
                
                return summary
                
            except Exception as e:
                logger.error(f"Error getting alert summary: {e}")
                return {"error": str(e)}

    def _get_price_target(self, insight: AlertInsight, current_price: float) -> Optional[float]:
        """Extract price target from insight reasoning"""
        # This is a simplified implementation
        # In a real system, you'd parse the reasoning more intelligently
        if insight.alert_type in [AlertType.PRICE_ABOVE, AlertType.PRICE_BELOW]:
            # Extract numbers from reasoning that could be price targets
            import re
            numbers = re.findall(r'\$?(\d+\.?\d*)', insight.reasoning)
            if numbers:
                target = float(numbers[0])
                if insight.alert_type == AlertType.PRICE_ABOVE and target > current_price:
                    return target
                elif insight.alert_type == AlertType.PRICE_BELOW and target < current_price:
                    return target
        return None

    def _get_percentage_change(self, insight: AlertInsight) -> Optional[float]:
        """Extract percentage change from insight reasoning"""
        if insight.alert_type in [AlertType.PERCENTAGE_GAIN, AlertType.PERCENTAGE_LOSS]:
            import re
            percentages = re.findall(r'(\d+\.?\d*)%', insight.reasoning)
            if percentages:
                return float(percentages[0])
        return None

    async def _get_recent_news(self, symbol: str) -> List[Dict[str, Any]]:
        """Get recent news for a symbol"""
        # This would integrate with the news monitoring service
        # For now, return mock data
        return [
            {
                "title": f"Recent news about {symbol}",
                "source": "Mock News",
                "published_at": datetime.now().isoformat(),
                "sentiment": 0.1
            }
        ]

# Example usage
async def main():
    # Initialize the copilot alert service
    copilot = CopilotAlertService(
        openai_api_key="your-openai-api-key",
        api_base_url="http://localhost:8000"
    )
    
    # Example user data
    user_id = "user123"
    symbols = ["AAPL", "GOOGL", "TSLA"]
    user_interests = ["technology", "growth stocks", "earnings announcements"]
    
    # Mock market data
    market_data = {
        "AAPL": {
            "price": 150.25,
            "high_52w": 180.50,
            "low_52w": 120.00,
            "volume": 50000000,
            "avg_volume": 45000000,
            "pe_ratio": 25.5,
            "market_cap": 2500000000000
        },
        "GOOGL": {
            "price": 2800.75,
            "high_52w": 3000.00,
            "low_52w": 2200.00,
            "volume": 2000000,
            "avg_volume": 1800000,
            "pe_ratio": 30.2,
            "market_cap": 1800000000000
        },
        "TSLA": {
            "price": 850.50,
            "high_52w": 900.00,
            "low_52w": 600.00,
            "volume": 30000000,
            "avg_volume": 25000000,
            "pe_ratio": 150.0,
            "market_cap": 800000000000
        }
    }
    
    # Set up intelligent alerts
    alert_results = await copilot.setup_intelligent_alerts(user_id, symbols, user_interests, market_data)
    print("Alert Setup Results:", json.dumps(alert_results, indent=2))
    
    # Set up news monitoring
    news_results = await copilot.setup_news_monitoring(user_id, symbols, user_interests)
    print("News Monitoring Results:", json.dumps(news_results, indent=2))
    
    # Get alert summary
    summary = await copilot.get_alert_summary(user_id)
    print("Alert Summary:", json.dumps(summary, indent=2))

if __name__ == "__main__":
    asyncio.run(main())
