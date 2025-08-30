"""
AI Copilot for Frontier Trading Platform

This module provides an AI assistant that analyzes market data, positions, and trading activity
to provide insights and suggestions while maintaining strict safety guardrails.
"""

import json
import logging
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Tuple, Any
from dataclasses import dataclass
from enum import Enum
import asyncio
from openai import AsyncOpenAI
import numpy as np
from dataclasses_json import dataclass_json

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class ConfidenceLevel(Enum):
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"

class RiskLevel(Enum):
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    EXTREME = "extreme"

@dataclass_json
@dataclass
class CopilotInsight:
    """Represents an AI-generated insight about the trading environment"""
    id: str
    type: str
    title: str
    description: str
    confidence: ConfidenceLevel
    risk_level: RiskLevel
    sources: List[str]
    timestamp: datetime
    actionable: bool
    suggested_actions: List[str]
    data_points: Dict[str, Any]
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            "id": self.id,
            "type": self.type,
            "title": self.title,
            "description": self.description,
            "confidence": self.confidence.value,
            "risk_level": self.risk_level.value,
            "sources": self.sources,
            "timestamp": self.timestamp.isoformat(),
            "actionable": self.actionable,
            "suggested_actions": self.suggested_actions,
            "data_points": self.data_points
        }

@dataclass_json
@dataclass
class MarketContext:
    """Context about current market conditions"""
    symbol: str
    current_price: float
    price_change_24h: float
    volume_24h: float
    volatility: float
    support_levels: List[float]
    resistance_levels: List[float]
    technical_indicators: Dict[str, float]
    news_sentiment: float
    market_sentiment: str

@dataclass_json
@dataclass
class PortfolioContext:
    """Context about current portfolio state"""
    total_value: float
    total_pnl: float
    daily_pnl: float
    positions: List[Dict[str, Any]]
    risk_metrics: Dict[str, float]
    recent_trades: List[Dict[str, Any]]
    open_orders: List[Dict[str, Any]]

class SafetyGuardrails:
    """Implements safety controls for AI Copilot"""
    
    def __init__(self):
        self.max_position_size_pct = 0.05  # 5% max position size
        self.max_daily_loss_pct = 0.02     # 2% max daily loss
        self.max_leverage = 2.0            # 2x max leverage
        self.forbidden_symbols = set()     # Symbols to avoid
        self.risk_thresholds = {
            "volatility": 0.5,             # High volatility threshold
            "drawdown": 0.1,               # 10% drawdown threshold
            "concentration": 0.3            # 30% concentration threshold
        }
    
    def validate_suggestion(self, insight: CopilotInsight, context: Dict[str, Any]) -> Tuple[bool, str]:
        """Validate if a suggestion is safe to present"""
        
        # Check risk level
        if insight.risk_level in [RiskLevel.HIGH, RiskLevel.EXTREME]:
            return False, f"Risk level {insight.risk_level.value} exceeds safety threshold"
        
        # Check confidence
        if insight.confidence == ConfidenceLevel.LOW:
            return False, "Confidence level too low for actionable suggestion"
        
        # Check portfolio context
        portfolio = context.get('portfolio', {})
        if portfolio:
            daily_pnl_pct = abs(portfolio.get('daily_pnl', 0) / portfolio.get('total_value', 1))
            if daily_pnl_pct > self.max_daily_loss_pct:
                return False, "Daily loss limit exceeded"
        
        # Check market context
        market = context.get('market', {})
        if market:
            volatility = market.get('volatility', 0)
            if volatility > self.risk_thresholds['volatility']:
                return False, "Market volatility too high"
        
        return True, "Suggestion validated"
    
    def add_forbidden_symbol(self, symbol: str):
        """Add symbol to forbidden list"""
        self.forbidden_symbols.add(symbol.upper())
    
    def remove_forbidden_symbol(self, symbol: str):
        """Remove symbol from forbidden list"""
        self.forbidden_symbols.discard(symbol.upper())

class CopilotAnalyzer:
    """Analyzes market and portfolio data to generate insights"""
    
    def __init__(self, openai_client: AsyncOpenAI):
        self.client = openai_client
        self.guardrails = SafetyGuardrails()
        
    async def analyze_market_context(self, market_data: Dict[str, Any]) -> List[CopilotInsight]:
        """Analyze market context and generate insights"""
        insights = []
        
        # Technical analysis insights
        technical_insights = await self._analyze_technical_indicators(market_data)
        insights.extend(technical_insights)
        
        # Volume analysis
        volume_insights = await self._analyze_volume_patterns(market_data)
        insights.extend(volume_insights)
        
        # Support/resistance analysis
        level_insights = await self._analyze_support_resistance(market_data)
        insights.extend(level_insights)
        
        return insights
    
    async def analyze_portfolio_context(self, portfolio_data: Dict[str, Any]) -> List[CopilotInsight]:
        """Analyze portfolio context and generate insights"""
        insights = []
        
        # Position concentration analysis
        concentration_insights = await self._analyze_position_concentration(portfolio_data)
        insights.extend(concentration_insights)
        
        # Risk metrics analysis
        risk_insights = await self._analyze_risk_metrics(portfolio_data)
        insights.extend(risk_insights)
        
        # Performance analysis
        performance_insights = await self._analyze_performance(portfolio_data)
        insights.extend(performance_insights)
        
        return insights
    
    async def generate_trading_suggestions(self, 
                                         market_context: MarketContext,
                                         portfolio_context: PortfolioContext) -> List[CopilotInsight]:
        """Generate trading suggestions based on current context"""
        
        # Prepare context for AI analysis
        context = {
            "market": market_context.__dict__,
            "portfolio": portfolio_context.__dict__,
            "timestamp": datetime.now().isoformat()
        }
        
        # Generate AI analysis
        prompt = self._build_analysis_prompt(context)
        
        try:
            response = await self.client.chat.completions.create(
                model="gpt-4",
                messages=[
                    {"role": "system", "content": self._get_system_prompt()},
                    {"role": "user", "content": prompt}
                ],
                temperature=0.3,
                max_tokens=1000
            )
            
            analysis = response.choices[0].message.content
            insights = self._parse_ai_response(analysis, context)
            
            # Apply safety guardrails
            validated_insights = []
            for insight in insights:
                is_safe, reason = self.guardrails.validate_suggestion(insight, context)
                if is_safe:
                    validated_insights.append(insight)
                else:
                    logger.warning(f"Insight rejected: {reason}")
            
            return validated_insights
            
        except Exception as e:
            logger.error(f"Error generating AI analysis: {e}")
            return []
    
    async def _analyze_technical_indicators(self, market_data: Dict[str, Any]) -> List[CopilotInsight]:
        """Analyze technical indicators"""
        insights = []
        
        indicators = market_data.get('technical_indicators', {})
        
        # RSI analysis
        rsi = indicators.get('rsi', 50)
        if rsi < 30:
            insights.append(CopilotInsight(
                id=f"rsi_oversold_{market_data['symbol']}",
                type="technical",
                title="Oversold Condition Detected",
                description=f"RSI of {rsi:.1f} indicates oversold conditions for {market_data['symbol']}",
                confidence=ConfidenceLevel.MEDIUM,
                risk_level=RiskLevel.LOW,
                sources=["RSI Technical Indicator"],
                timestamp=datetime.now(),
                actionable=True,
                suggested_actions=["Consider buying opportunities", "Monitor for reversal signals"],
                data_points={"rsi": rsi, "threshold": 30}
            ))
        elif rsi > 70:
            insights.append(CopilotInsight(
                id=f"rsi_overbought_{market_data['symbol']}",
                type="technical",
                title="Overbought Condition Detected",
                description=f"RSI of {rsi:.1f} indicates overbought conditions for {market_data['symbol']}",
                confidence=ConfidenceLevel.MEDIUM,
                risk_level=RiskLevel.MEDIUM,
                sources=["RSI Technical Indicator"],
                timestamp=datetime.now(),
                actionable=True,
                suggested_actions=["Consider taking profits", "Monitor for reversal signals"],
                data_points={"rsi": rsi, "threshold": 70}
            ))
        
        return insights
    
    async def _analyze_volume_patterns(self, market_data: Dict[str, Any]) -> List[CopilotInsight]:
        """Analyze volume patterns"""
        insights = []
        
        volume_24h = market_data.get('volume_24h', 0)
        avg_volume = market_data.get('average_volume', volume_24h)
        
        if volume_24h > avg_volume * 1.5:
            insights.append(CopilotInsight(
                id=f"high_volume_{market_data['symbol']}",
                type="volume",
                title="High Volume Activity",
                description=f"Volume is {volume_24h/avg_volume:.1f}x above average for {market_data['symbol']}",
                confidence=ConfidenceLevel.HIGH,
                risk_level=RiskLevel.MEDIUM,
                sources=["Volume Analysis"],
                timestamp=datetime.now(),
                actionable=True,
                suggested_actions=["Monitor for price breakouts", "Check for news catalysts"],
                data_points={"current_volume": volume_24h, "avg_volume": avg_volume, "ratio": volume_24h/avg_volume}
            ))
        
        return insights
    
    async def _analyze_support_resistance(self, market_data: Dict[str, Any]) -> List[CopilotInsight]:
        """Analyze support and resistance levels"""
        insights = []
        
        current_price = market_data.get('current_price', 0)
        support_levels = market_data.get('support_levels', [])
        resistance_levels = market_data.get('resistance_levels', [])
        
        # Check if price is near support
        for support in support_levels:
            if abs(current_price - support) / support < 0.02:  # Within 2%
                insights.append(CopilotInsight(
                    id=f"support_test_{market_data['symbol']}",
                    type="technical",
                    title="Support Level Test",
                    description=f"{market_data['symbol']} is testing support at ${support:.2f}",
                    confidence=ConfidenceLevel.MEDIUM,
                    risk_level=RiskLevel.LOW,
                    sources=["Support/Resistance Analysis"],
                    timestamp=datetime.now(),
                    actionable=True,
                    suggested_actions=["Monitor for bounce", "Set stop loss below support"],
                    data_points={"current_price": current_price, "support_level": support}
                ))
        
        # Check if price is near resistance
        for resistance in resistance_levels:
            if abs(current_price - resistance) / resistance < 0.02:  # Within 2%
                insights.append(CopilotInsight(
                    id=f"resistance_test_{market_data['symbol']}",
                    type="technical",
                    title="Resistance Level Test",
                    description=f"{market_data['symbol']} is testing resistance at ${resistance:.2f}",
                    confidence=ConfidenceLevel.MEDIUM,
                    risk_level=RiskLevel.MEDIUM,
                    sources=["Support/Resistance Analysis"],
                    timestamp=datetime.now(),
                    actionable=True,
                    suggested_actions=["Monitor for breakout", "Consider taking profits"],
                    data_points={"current_price": current_price, "resistance_level": resistance}
                ))
        
        return insights
    
    async def _analyze_position_concentration(self, portfolio_data: Dict[str, Any]) -> List[CopilotInsight]:
        """Analyze position concentration risks"""
        insights = []
        
        positions = portfolio_data.get('positions', [])
        total_value = portfolio_data.get('total_value', 1)
        
        for position in positions:
            position_value = position.get('market_value', 0)
            concentration = position_value / total_value
            
            if concentration > 0.2:  # More than 20% in single position
                insights.append(CopilotInsight(
                    id=f"concentration_risk_{position['symbol']}",
                    type="risk",
                    title="High Position Concentration",
                    description=f"{position['symbol']} represents {concentration:.1%} of portfolio",
                    confidence=ConfidenceLevel.HIGH,
                    risk_level=RiskLevel.HIGH,
                    sources=["Portfolio Analysis"],
                    timestamp=datetime.now(),
                    actionable=True,
                    suggested_actions=["Consider reducing position size", "Diversify portfolio"],
                    data_points={"symbol": position['symbol'], "concentration": concentration}
                ))
        
        return insights
    
    async def _analyze_risk_metrics(self, portfolio_data: Dict[str, Any]) -> List[CopilotInsight]:
        """Analyze portfolio risk metrics"""
        insights = []
        
        risk_metrics = portfolio_data.get('risk_metrics', {})
        
        # Check drawdown
        drawdown = risk_metrics.get('current_drawdown', 0)
        if drawdown > 0.1:  # More than 10% drawdown
            insights.append(CopilotInsight(
                id="high_drawdown_warning",
                type="risk",
                title="High Portfolio Drawdown",
                description=f"Portfolio is experiencing {drawdown:.1%} drawdown",
                confidence=ConfidenceLevel.HIGH,
                risk_level=RiskLevel.HIGH,
                sources=["Risk Metrics"],
                timestamp=datetime.now(),
                actionable=True,
                suggested_actions=["Review risk management", "Consider reducing exposure"],
                data_points={"drawdown": drawdown}
            ))
        
        # Check leverage
        leverage = risk_metrics.get('leverage', 1.0)
        if leverage > 1.5:
            insights.append(CopilotInsight(
                id="high_leverage_warning",
                type="risk",
                title="High Leverage Warning",
                description=f"Portfolio leverage is {leverage:.1f}x",
                confidence=ConfidenceLevel.HIGH,
                risk_level=RiskLevel.HIGH,
                sources=["Risk Metrics"],
                timestamp=datetime.now(),
                actionable=True,
                suggested_actions=["Reduce leverage", "Monitor margin requirements"],
                data_points={"leverage": leverage}
            ))
        
        return insights
    
    async def _analyze_performance(self, portfolio_data: Dict[str, Any]) -> List[CopilotInsight]:
        """Analyze portfolio performance"""
        insights = []
        
        daily_pnl = portfolio_data.get('daily_pnl', 0)
        total_pnl = portfolio_data.get('total_pnl', 0)
        
        # Check for significant gains
        if daily_pnl > 0 and abs(daily_pnl) > portfolio_data.get('total_value', 1) * 0.01:
            insights.append(CopilotInsight(
                id="strong_daily_performance",
                type="performance",
                title="Strong Daily Performance",
                description=f"Portfolio gained {daily_pnl:.2f} today",
                confidence=ConfidenceLevel.HIGH,
                risk_level=RiskLevel.LOW,
                sources=["Performance Analysis"],
                timestamp=datetime.now(),
                actionable=True,
                suggested_actions=["Consider taking partial profits", "Review winning positions"],
                data_points={"daily_pnl": daily_pnl}
            ))
        
        return insights
    
    def _build_analysis_prompt(self, context: Dict[str, Any]) -> str:
        """Build prompt for AI analysis"""
        return f"""
        Analyze the following trading context and provide insights:
        
        Market Context:
        {json.dumps(context['market'], indent=2)}
        
        Portfolio Context:
        {json.dumps(context['portfolio'], indent=2)}
        
        Please provide:
        1. Key market insights
        2. Portfolio risk assessment
        3. Trading opportunities
        4. Risk warnings
        
        Focus on actionable insights with clear confidence levels and risk assessments.
        """
    
    def _get_system_prompt(self) -> str:
        """Get system prompt for AI"""
        return """
        You are an AI trading assistant for the Frontier Trading Platform. Your role is to:
        
        1. Analyze market data and portfolio information
        2. Provide insights with confidence levels and risk assessments
        3. Suggest actionable trading ideas with clear reasoning
        4. Always prioritize risk management and safety
        5. Never make specific buy/sell recommendations without proper context
        6. Always include sources and data points for your analysis
        
        Format your responses as structured insights with:
        - Clear titles and descriptions
        - Confidence levels (low/medium/high)
        - Risk levels (low/medium/high/extreme)
        - Actionable suggestions
        - Supporting data points
        """
    
    def _parse_ai_response(self, response: str, context: Dict[str, Any]) -> List[CopilotInsight]:
        """Parse AI response into structured insights"""
        insights = []
        
        # This is a simplified parser - in production, you'd want more robust parsing
        try:
            # Extract insights from AI response
            # For now, return empty list - implement based on your AI response format
            pass
        except Exception as e:
            logger.error(f"Error parsing AI response: {e}")
        
        return insights

class CopilotService:
    """Main service for AI Copilot functionality"""
    
    def __init__(self, openai_api_key: str):
        self.client = AsyncOpenAI(api_key=openai_api_key)
        self.analyzer = CopilotAnalyzer(self.client)
        self.insight_history: List[CopilotInsight] = []
        
    async def get_insights(self, 
                          market_data: Dict[str, Any],
                          portfolio_data: Dict[str, Any]) -> List[CopilotInsight]:
        """Get AI insights for current market and portfolio context"""
        
        # Create context objects
        market_context = MarketContext(**market_data)
        portfolio_context = PortfolioContext(**portfolio_data)
        
        # Generate insights
        insights = await self.analyzer.generate_trading_suggestions(
            market_context, portfolio_context
        )
        
        # Add to history
        self.insight_history.extend(insights)
        
        # Keep only recent insights (last 24 hours)
        cutoff_time = datetime.now() - timedelta(hours=24)
        self.insight_history = [
            insight for insight in self.insight_history 
            if insight.timestamp > cutoff_time
        ]
        
        return insights
    
    async def get_insight_history(self, hours: int = 24) -> List[CopilotInsight]:
        """Get insight history for the specified time period"""
        cutoff_time = datetime.now() - timedelta(hours=hours)
        return [
            insight for insight in self.insight_history 
            if insight.timestamp > cutoff_time
        ]
    
    def add_safety_rule(self, rule_type: str, rule_config: Dict[str, Any]):
        """Add custom safety rule"""
        if rule_type == "forbidden_symbol":
            self.analyzer.guardrails.add_forbidden_symbol(rule_config['symbol'])
        elif rule_type == "max_position_size":
            self.analyzer.guardrails.max_position_size_pct = rule_config['percentage']
        elif rule_type == "max_daily_loss":
            self.analyzer.guardrails.max_daily_loss_pct = rule_config['percentage']
    
    def get_safety_config(self) -> Dict[str, Any]:
        """Get current safety configuration"""
        return {
            "max_position_size_pct": self.analyzer.guardrails.max_position_size_pct,
            "max_daily_loss_pct": self.analyzer.guardrails.max_daily_loss_pct,
            "max_leverage": self.analyzer.guardrails.max_leverage,
            "forbidden_symbols": list(self.analyzer.guardrails.forbidden_symbols),
            "risk_thresholds": self.analyzer.guardrails.risk_thresholds
        }
