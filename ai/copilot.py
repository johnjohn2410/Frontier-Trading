import asyncio
import aiohttp
import json
import logging
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass, asdict
from decimal import Decimal
import re
from enum import Enum

# ============================================================================
# ENHANCED COPILOT SUGGESTION MODEL
# ============================================================================

class CopilotActionType(Enum):
    BUY = "buy"
    SELL = "sell"
    HOLD = "hold"
    WATCH = "watch"
    ALERT = "alert"

@dataclass
class RiskImpact:
    estimated_drawdown: float
    bp_usage: float  # Percentage of buying power
    max_loss: float
    risk_reward_ratio: Optional[float] = None

@dataclass
class CopilotFeatures:
    volume_z_score: Optional[float] = None
    rsi: Optional[float] = None
    news_sentiment: Optional[float] = None
    technical_signals: List[str] = None
    fundamental_metrics: Dict[str, Any] = None
    
    def __post_init__(self):
        if self.technical_signals is None:
            self.technical_signals = []
        if self.fundamental_metrics is None:
            self.fundamental_metrics = {}

@dataclass
class WhatIfAnalysis:
    quantity: float
    price: float
    estimated_cost: float
    estimated_fees: float
    potential_pnl: float
    risk_metrics: Dict[str, Any] = None
    
    def __post_init__(self):
        if self.risk_metrics is None:
            self.risk_metrics = {}

@dataclass
class GuardrailCheck:
    max_position_ok: bool
    daily_loss_ok: bool
    market_hours_ok: bool
    pattern_day_trader_ok: bool
    risk_limits_ok: bool
    violations: List[str] = None
    
    def __post_init__(self):
        if self.violations is None:
            self.violations = []

@dataclass
class ComplianceInfo:
    disclaimer: str
    requires_confirmation: bool
    not_financial_advice: bool
    risk_disclosure: str

@dataclass
class CopilotSuggestion:
    suggestion_id: str
    user_id: str
    symbol: str
    suggestion: str
    action_type: CopilotActionType
    confidence: float  # 0.0 to 1.0
    risk_impact: RiskImpact
    features: CopilotFeatures
    what_if: WhatIfAnalysis
    guardrails: GuardrailCheck
    compliance: ComplianceInfo
    timestamp: datetime = None
    
    def __post_init__(self):
        if self.timestamp is None:
            self.timestamp = datetime.utcnow()

# ============================================================================
# ENHANCED ALERT ANALYZER
# ============================================================================

class AlertAnalyzer:
    def __init__(self, openai_client):
        self.openai_client = openai_client
        self.logger = logging.getLogger(__name__)
    
    async def analyze_stock_for_alerts(self, symbol: str, user_id: str, 
                                     market_data: Dict[str, Any], 
                                     news_data: List[Dict[str, Any]],
                                     account_data: Dict[str, Any]) -> CopilotSuggestion:
        """Analyze stock and generate intelligent alert suggestions."""
        
        # Build context for AI analysis
        context = self._build_analysis_context(symbol, market_data, news_data, account_data)
        
        # Get AI analysis
        analysis = await self._get_ai_analysis(context)
        
        # Parse AI response and create structured suggestion
        suggestion = self._parse_ai_suggestion(analysis, symbol, user_id, market_data, account_data)
        
        return suggestion
    
    def _build_analysis_context(self, symbol: str, market_data: Dict[str, Any], 
                               news_data: List[Dict[str, Any]], 
                               account_data: Dict[str, Any]) -> str:
        """Build comprehensive context for AI analysis."""
        
        context_parts = [
            f"Stock: {symbol}",
            f"Current Price: ${market_data.get('price', 0):.2f}",
            f"Volume: {market_data.get('volume', 0):,}",
            f"24h Change: {market_data.get('change_percent', 0):.2f}%",
        ]
        
        # Add technical indicators if available
        if 'rsi' in market_data:
            context_parts.append(f"RSI: {market_data['rsi']:.1f}")
        if 'volume_avg' in market_data:
            volume_ratio = market_data.get('volume', 0) / market_data['volume_avg']
            context_parts.append(f"Volume Ratio: {volume_ratio:.2f}x average")
        
        # Add news sentiment
        if news_data:
            avg_sentiment = sum(n.get('sentiment', 0) for n in news_data) / len(news_data)
            context_parts.append(f"News Sentiment: {avg_sentiment:.2f} (-1 to 1)")
        
        # Add account context
        context_parts.extend([
            f"Available Cash: ${account_data.get('cash', 0):,.2f}",
            f"Current Equity: ${account_data.get('equity', 0):,.2f}",
            f"Buying Power: ${account_data.get('buying_power', 0):,.2f}",
        ])
        
        return "\n".join(context_parts)
    
    async def _get_ai_analysis(self, context: str) -> Dict[str, Any]:
        """Get AI analysis using OpenAI GPT-4."""
        
        prompt = f"""
You are an AI trading assistant analyzing a stock for potential trading opportunities. 
Your goal is to provide clear, actionable suggestions with confidence levels and risk assessments.

Context:
{context}

Please analyze this stock and provide a structured response in JSON format with the following fields:

{{
    "action_type": "buy|sell|hold|watch|alert",
    "confidence": 0.0-1.0,
    "reasoning": "Clear explanation of why this action is suggested",
    "risk_level": "low|medium|high",
    "key_factors": [
        "Factor 1: description",
        "Factor 2: description"
    ],
    "technical_signals": [
        "Signal 1",
        "Signal 2"
    ],
    "suggested_quantity": 0,
    "suggested_price": 0.0,
    "estimated_cost": 0.0,
    "potential_upside": "percentage or dollar amount",
    "potential_downside": "percentage or dollar amount",
    "risk_reward_ratio": 0.0,
    "time_horizon": "short|medium|long",
    "stop_loss_suggestion": 0.0,
    "take_profit_suggestion": 0.0
}}

Focus on:
1. Clear, explainable reasoning
2. Conservative confidence levels
3. Risk-aware suggestions
4. Specific, actionable recommendations
5. Compliance with trading best practices

Remember: This is for educational purposes only. Always suggest appropriate position sizing and risk management.
"""
        
        try:
            response = await self.openai_client.chat.completions.create(
                model="gpt-4",
                messages=[
                    {"role": "system", "content": "You are a professional trading assistant focused on risk management and clear explanations."},
                    {"role": "user", "content": prompt}
                ],
                temperature=0.3,
                max_tokens=1000
            )
            
            content = response.choices[0].message.content
            return json.loads(content)
            
        except Exception as e:
            self.logger.error(f"AI analysis failed: {e}")
            return self._get_fallback_analysis()
    
    def _get_fallback_analysis(self) -> Dict[str, Any]:
        """Fallback analysis when AI fails."""
        return {
            "action_type": "watch",
            "confidence": 0.3,
            "reasoning": "Insufficient data for confident recommendation",
            "risk_level": "medium",
            "key_factors": ["Limited market data available"],
            "technical_signals": [],
            "suggested_quantity": 0,
            "suggested_price": 0.0,
            "estimated_cost": 0.0,
            "potential_upside": "Unknown",
            "potential_downside": "Unknown",
            "risk_reward_ratio": 1.0,
            "time_horizon": "short",
            "stop_loss_suggestion": 0.0,
            "take_profit_suggestion": 0.0
        }
    
    def _parse_ai_suggestion(self, analysis: Dict[str, Any], symbol: str, user_id: str,
                           market_data: Dict[str, Any], account_data: Dict[str, Any]) -> CopilotSuggestion:
        """Parse AI analysis into structured CopilotSuggestion."""
        
        import uuid
        
        # Extract values with defaults
        action_type = CopilotActionType(analysis.get("action_type", "watch"))
        confidence = float(analysis.get("confidence", 0.3))
        suggested_quantity = float(analysis.get("suggested_quantity", 0))
        suggested_price = float(analysis.get("suggested_price", market_data.get("price", 0)))
        
        # Calculate risk impact
        estimated_cost = suggested_quantity * suggested_price
        bp_usage = (estimated_cost / account_data.get("buying_power", 1)) * 100
        
        risk_impact = RiskImpact(
            estimated_drawdown=float(analysis.get("potential_downside", 0)),
            bp_usage=min(bp_usage, 100.0),
            max_loss=estimated_cost * (float(analysis.get("potential_downside", 0)) / 100),
            risk_reward_ratio=float(analysis.get("risk_reward_ratio", 1.0))
        )
        
        # Build features
        features = CopilotFeatures(
            volume_z_score=market_data.get("volume_z_score"),
            rsi=market_data.get("rsi"),
            news_sentiment=market_data.get("news_sentiment"),
            technical_signals=analysis.get("technical_signals", []),
            fundamental_metrics=market_data.get("fundamental_metrics", {})
        )
        
        # Calculate what-if analysis
        what_if = WhatIfAnalysis(
            quantity=suggested_quantity,
            price=suggested_price,
            estimated_cost=estimated_cost,
            estimated_fees=estimated_cost * 0.005,  # 0.5% estimated fees
            potential_pnl=estimated_cost * (float(analysis.get("potential_upside", 0)) / 100),
            risk_metrics={
                "stop_loss": float(analysis.get("stop_loss_suggestion", 0)),
                "take_profit": float(analysis.get("take_profit_suggestion", 0)),
                "time_horizon": analysis.get("time_horizon", "short")
            }
        )
        
        # Check guardrails
        guardrails = self._check_guardrails(account_data, estimated_cost, symbol)
        
        # Compliance info
        compliance = ComplianceInfo(
            disclaimer="This analysis is for educational purposes only",
            requires_confirmation=True,
            not_financial_advice=True,
            risk_disclosure="Trading involves risk of loss. Past performance does not guarantee future results."
        )
        
        return CopilotSuggestion(
            suggestion_id=str(uuid.uuid4()),
            user_id=user_id,
            symbol=symbol,
            suggestion=analysis.get("reasoning", "No specific recommendation"),
            action_type=action_type,
            confidence=confidence,
            risk_impact=risk_impact,
            features=features,
            what_if=what_if,
            guardrails=guardrails,
            compliance=compliance
        )
    
    def _check_guardrails(self, account_data: Dict[str, Any], estimated_cost: float, symbol: str) -> GuardrailCheck:
        """Check if the suggested trade meets risk guardrails."""
        
        violations = []
        
        # Check buying power
        buying_power = account_data.get("buying_power", 0)
        if estimated_cost > buying_power:
            violations.append("Insufficient buying power")
        
        # Check position limits (example: max 5% of portfolio per position)
        equity = account_data.get("equity", 1)
        position_limit = equity * 0.05
        if estimated_cost > position_limit:
            violations.append("Position size exceeds 5% portfolio limit")
        
        # Check daily loss limits
        daily_pnl = account_data.get("daily_pnl", 0)
        daily_limit = account_data.get("daily_loss_limit", -1000)
        if daily_pnl < daily_limit:
            violations.append("Daily loss limit reached")
        
        # Check market hours (simplified)
        now = datetime.now()
        market_hours_ok = 9 <= now.hour < 16  # Simplified market hours
        
        return GuardrailCheck(
            max_position_ok=len(violations) == 0,
            daily_loss_ok=daily_pnl >= daily_limit,
            market_hours_ok=market_hours_ok,
            pattern_day_trader_ok=True,  # Would check PDT status
            risk_limits_ok=len(violations) == 0,
            violations=violations
        )

# ============================================================================
# ENHANCED COPILOT ALERT SERVICE
# ============================================================================

class CopilotAlertService:
    def __init__(self, alert_manager: 'AlertManager', analyzer: AlertAnalyzer):
        self.alert_manager = alert_manager
        self.analyzer = analyzer
        self.logger = logging.getLogger(__name__)
    
    async def setup_intelligent_alerts(self, user_id: str, symbol: str, 
                                     market_data: Dict[str, Any],
                                     news_data: List[Dict[str, Any]],
                                     account_data: Dict[str, Any]) -> CopilotSuggestion:
        """Set up intelligent alerts based on AI analysis."""
        
        # Get AI suggestion
        suggestion = await self.analyzer.analyze_stock_for_alerts(
            symbol, user_id, market_data, news_data, account_data
        )
        
        # Create alerts based on suggestion
        await self._create_alerts_from_suggestion(suggestion)
        
        return suggestion
    
    async def _create_alerts_from_suggestion(self, suggestion: CopilotSuggestion):
        """Create specific alerts based on AI suggestion."""
        
        if suggestion.action_type == CopilotActionType.ALERT:
            # Create price alerts
            if suggestion.what_if.risk_metrics.get("stop_loss"):
                await self.alert_manager.create_price_alert(
                    user_id=suggestion.user_id,
                    symbol=suggestion.symbol,
                    alert_type="stop_loss",
                    trigger_price=suggestion.what_if.risk_metrics["stop_loss"],
                    message=f"Stop loss triggered for {suggestion.symbol}"
                )
            
            if suggestion.what_if.risk_metrics.get("take_profit"):
                await self.alert_manager.create_price_alert(
                    user_id=suggestion.user_id,
                    symbol=suggestion.symbol,
                    alert_type="take_profit",
                    trigger_price=suggestion.what_if.risk_metrics["take_profit"],
                    message=f"Take profit target reached for {suggestion.symbol}"
                )
        
        # Create news alerts for the symbol
        await self.alert_manager.create_news_alert(
            user_id=suggestion.user_id,
            symbol=suggestion.symbol,
            keywords=[suggestion.symbol, "earnings", "news", "announcement"],
            message=f"News alert for {suggestion.symbol}"
        )

# ============================================================================
# ENHANCED ALERT MANAGER
# ============================================================================

class AlertManager:
    def __init__(self, base_url: str = "http://localhost:8002"):
        self.base_url = base_url
        self.session = None
        self.logger = logging.getLogger(__name__)
    
    async def __aenter__(self):
        self.session = aiohttp.ClientSession()
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()
    
    async def create_price_alert(self, user_id: str, symbol: str, alert_type: str,
                               trigger_price: float, message: str) -> Dict[str, Any]:
        """Create a price-based alert."""
        
        payload = {
            "user_id": user_id,
            "symbol": symbol,
            "alert_type": alert_type,
            "trigger_price": trigger_price,
            "message": message,
            "active": True
        }
        
        async with self.session.post(f"{self.base_url}/api/alerts/price", json=payload) as response:
            return await response.json()
    
    async def create_news_alert(self, user_id: str, symbol: str, keywords: List[str],
                              message: str) -> Dict[str, Any]:
        """Create a news-based alert."""
        
        payload = {
            "user_id": user_id,
            "symbol": symbol,
            "keywords": keywords,
            "message": message,
            "active": True
        }
        
        async with self.session.post(f"{self.base_url}/api/alerts/news", json=payload) as response:
            return await response.json()
    
    async def get_alert_insights(self, user_id: str, symbol: str) -> List[Dict[str, Any]]:
        """Get AI-powered insights for alerts."""
        
        async with self.session.get(f"{self.base_url}/api/alerts/insights/{user_id}/{symbol}") as response:
            return await response.json()

# ============================================================================
# MAIN COPILOT CLASS
# ============================================================================

class FrontierCopilot:
    def __init__(self, openai_api_key: str, alert_service_url: str = "http://localhost:8002"):
        self.openai_client = openai.OpenAI(api_key=openai_api_key)
        self.alert_manager = AlertManager(alert_service_url)
        self.analyzer = AlertAnalyzer(self.openai_client)
        self.alert_service = CopilotAlertService(self.alert_manager, self.analyzer)
        self.logger = logging.getLogger(__name__)
    
    async def analyze_stock(self, user_id: str, symbol: str, 
                          market_data: Dict[str, Any],
                          news_data: List[Dict[str, Any]] = None,
                          account_data: Dict[str, Any] = None) -> CopilotSuggestion:
        """Main method to analyze a stock and provide suggestions."""
        
        if news_data is None:
            news_data = []
        if account_data is None:
            account_data = {"cash": 10000, "equity": 10000, "buying_power": 10000}
        
        async with self.alert_manager:
            suggestion = await self.alert_service.setup_intelligent_alerts(
                user_id, symbol, market_data, news_data, account_data
            )
        
        return suggestion
    
    async def get_explainable_suggestion(self, suggestion: CopilotSuggestion) -> Dict[str, Any]:
        """Convert suggestion to explainable format for UI."""
        
        return {
            "type": "copilot.suggestion.v1",
            "symbol": suggestion.symbol,
            "suggestion": suggestion.suggestion,
            "action_type": suggestion.action_type.value,
            "confidence": suggestion.confidence,
            "features": {
                "volume_z_score": suggestion.features.volume_z_score,
                "rsi": suggestion.features.rsi,
                "news_sentiment": suggestion.features.news_sentiment,
                "technical_signals": suggestion.features.technical_signals
            },
            "what_if": {
                "quantity": suggestion.what_if.quantity,
                "price": suggestion.what_if.price,
                "estimated_cost": suggestion.what_if.estimated_cost,
                "potential_pnl": suggestion.what_if.potential_pnl
            },
            "guardrails": {
                "max_position_ok": suggestion.guardrails.max_position_ok,
                "daily_loss_ok": suggestion.guardrails.daily_loss_ok,
                "market_hours_ok": suggestion.guardrails.market_hours_ok,
                "violations": suggestion.guardrails.violations
            },
            "compliance": {
                "disclaimer": suggestion.compliance.disclaimer,
                "requires_confirmation": suggestion.compliance.requires_confirmation,
                "risk_disclosure": suggestion.compliance.risk_disclosure
            }
        }

# ============================================================================
# EXAMPLE USAGE
# ============================================================================

async def main():
    """Example usage of the enhanced Copilot."""
    
    # Initialize copilot
    copilot = FrontierCopilot(
        openai_api_key="your-openai-api-key",
        alert_service_url="http://localhost:8002"
    )
    
    # Example market data
    market_data = {
        "price": 150.25,
        "volume": 1000000,
        "change_percent": 2.5,
        "rsi": 65.5,
        "volume_avg": 800000,
        "volume_z_score": 1.25,
        "news_sentiment": 0.3
    }
    
    # Example account data
    account_data = {
        "cash": 50000,
        "equity": 100000,
        "buying_power": 50000,
        "daily_pnl": -500,
        "daily_loss_limit": -1000
    }
    
    # Analyze stock
    suggestion = await copilot.analyze_stock(
        user_id="user123",
        symbol="AAPL",
        market_data=market_data,
        account_data=account_data
    )
    
    # Get explainable format
    explainable = await copilot.get_explainable_suggestion(suggestion)
    
    print("Copilot Suggestion:")
    print(json.dumps(explainable, indent=2, default=str))

if __name__ == "__main__":
    asyncio.run(main())
