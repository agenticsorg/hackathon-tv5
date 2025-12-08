#!/usr/bin/env python3
"""
Entertainment Discovery Multi-Agent System
Demonstrates collaborative agents solving the "what to watch" problem
"""

import asyncio
import json
from datetime import datetime
from typing import Dict, List, Any


class Agent:
    """Base Agent class"""
    def __init__(self, name: str, role: str):
        self.name = name
        self.role = role
        self.memory: List[Dict] = []

    def log(self, message: str):
        """Log agent activity"""
        timestamp = datetime.now().strftime("%H:%M:%S")
        print(f"[{timestamp}] 🤖 {self.name} ({self.role}): {message}")

    def remember(self, key: str, value: Any):
        """Store information in memory"""
        self.memory.append({"key": key, "value": value, "timestamp": datetime.now()})

    async def execute(self, task: str) -> Dict[str, Any]:
        """Execute agent task - to be overridden"""
        raise NotImplementedError


class ResearchAgent(Agent):
    """Agent that researches user preferences and content availability"""

    def __init__(self):
        super().__init__("ResearchBot", "Content Research")

    async def execute(self, task: str) -> Dict[str, Any]:
        """Research content based on user preferences"""
        self.log(f"Researching: {task}")

        # Simulate research delay
        await asyncio.sleep(1)

        # Simulate finding content across platforms
        results = {
            "netflix": [
                {"title": "Stranger Things", "genre": "sci-fi", "rating": 8.7},
                {"title": "The Crown", "genre": "drama", "rating": 8.6},
                {"title": "Arcane", "genre": "animation", "rating": 9.0}
            ],
            "disney_plus": [
                {"title": "The Mandalorian", "genre": "sci-fi", "rating": 8.7},
                {"title": "Loki", "genre": "sci-fi", "rating": 8.2}
            ],
            "hbo_max": [
                {"title": "House of the Dragon", "genre": "fantasy", "rating": 8.5},
                {"title": "The Last of Us", "genre": "drama", "rating": 8.8}
            ]
        }

        self.remember("research_results", results)
        self.log(f"Found {sum(len(v) for v in results.values())} shows across {len(results)} platforms")

        return {
            "status": "success",
            "data": results,
            "agent": self.name
        }


class AnalysisAgent(Agent):
    """Agent that analyzes content and matches preferences"""

    def __init__(self):
        super().__init__("AnalyzerBot", "Content Analysis")

    async def execute(self, research_data: Dict[str, Any]) -> Dict[str, Any]:
        """Analyze research data and rank recommendations"""
        self.log("Analyzing content and matching preferences...")

        # Simulate analysis delay
        await asyncio.sleep(1)

        # Extract all shows
        all_shows = []
        data = research_data.get("data", {})
        for platform, shows in data.items():
            for show in shows:
                show["platform"] = platform
                all_shows.append(show)

        # Sort by rating
        ranked = sorted(all_shows, key=lambda x: x["rating"], reverse=True)

        # Analyze patterns
        genres = {}
        for show in all_shows:
            genre = show["genre"]
            genres[genre] = genres.get(genre, 0) + 1

        analysis = {
            "top_recommendations": ranked[:3],
            "total_analyzed": len(all_shows),
            "genre_distribution": genres,
            "average_rating": sum(s["rating"] for s in all_shows) / len(all_shows)
        }

        self.remember("analysis", analysis)
        self.log(f"Analyzed {len(all_shows)} shows, top pick: {ranked[0]['title']} ({ranked[0]['rating']}/10)")

        return {
            "status": "success",
            "data": analysis,
            "agent": self.name
        }


class RecommendationAgent(Agent):
    """Agent that generates final recommendations with explanations"""

    def __init__(self):
        super().__init__("RecommendBot", "Recommendation Generation")

    async def execute(self, analysis_data: Dict[str, Any]) -> Dict[str, Any]:
        """Generate personalized recommendations"""
        self.log("Generating personalized recommendations...")

        # Simulate processing delay
        await asyncio.sleep(1)

        data = analysis_data.get("data", {})
        top_picks = data.get("top_recommendations", [])

        recommendations = []
        for i, show in enumerate(top_picks, 1):
            rec = {
                "rank": i,
                "title": show["title"],
                "platform": show["platform"],
                "genre": show["genre"],
                "rating": show["rating"],
                "reason": self._generate_reason(show, data)
            }
            recommendations.append(rec)

        self.remember("recommendations", recommendations)
        self.log(f"Generated {len(recommendations)} personalized recommendations")

        return {
            "status": "success",
            "data": {
                "recommendations": recommendations,
                "insights": {
                    "total_options": data.get("total_analyzed", 0),
                    "avg_quality": data.get("average_rating", 0),
                    "popular_genres": data.get("genre_distribution", {})
                }
            },
            "agent": self.name
        }

    def _generate_reason(self, show: Dict, analysis: Dict) -> str:
        """Generate recommendation reason"""
        reasons = [
            f"Highly rated at {show['rating']}/10",
            f"Popular {show['genre']} genre",
            f"Available on {show['platform'].replace('_', ' ').title()}"
        ]
        return " • ".join(reasons)


class CoordinatorAgent(Agent):
    """Coordinator agent that orchestrates the multi-agent workflow"""

    def __init__(self):
        super().__init__("Coordinator", "Workflow Orchestration")
        self.agents = {
            "research": ResearchAgent(),
            "analysis": AnalysisAgent(),
            "recommendation": RecommendationAgent()
        }

    async def execute(self, user_query: str) -> Dict[str, Any]:
        """Coordinate the multi-agent workflow"""
        self.log("=" * 60)
        self.log(f"Starting multi-agent workflow for query: '{user_query}'")
        self.log("=" * 60)

        try:
            # Step 1: Research
            self.log("Step 1: Delegating to Research Agent")
            research_result = await self.agents["research"].execute(user_query)

            # Step 2: Analysis
            self.log("\nStep 2: Delegating to Analysis Agent")
            analysis_result = await self.agents["analysis"].execute(research_result)

            # Step 3: Recommendations
            self.log("\nStep 3: Delegating to Recommendation Agent")
            recommendation_result = await self.agents["recommendation"].execute(analysis_result)

            # Final summary
            self.log("\n" + "=" * 60)
            self.log("Multi-agent workflow completed successfully!")
            self.log("=" * 60)

            return {
                "status": "success",
                "query": user_query,
                "results": recommendation_result["data"],
                "agents_involved": [
                    self.agents["research"].name,
                    self.agents["analysis"].name,
                    self.agents["recommendation"].name
                ]
            }

        except Exception as e:
            self.log(f"ERROR: Workflow failed - {str(e)}")
            return {
                "status": "error",
                "error": str(e)
            }


def display_recommendations(result: Dict[str, Any]):
    """Display recommendations in a user-friendly format"""
    print("\n" + "=" * 60)
    print("🎬 ENTERTAINMENT DISCOVERY RESULTS")
    print("=" * 60)

    if result["status"] == "error":
        print(f"❌ Error: {result['error']}")
        return

    recommendations = result["results"]["recommendations"]
    insights = result["results"]["insights"]

    print(f"\n📊 Analyzed {insights['total_options']} options")
    print(f"⭐ Average rating: {insights['avg_quality']:.1f}/10")
    print(f"🎭 Popular genres: {', '.join(insights['popular_genres'].keys())}")

    print(f"\n🏆 TOP RECOMMENDATIONS:\n")

    for rec in recommendations:
        print(f"{rec['rank']}. {rec['title']}")
        print(f"   Platform: {rec['platform'].replace('_', ' ').title()}")
        print(f"   Genre: {rec['genre'].title()} | Rating: {rec['rating']}/10")
        print(f"   Why: {rec['reason']}")
        print()

    print("=" * 60)
    print(f"✅ Agents involved: {', '.join(result['agents_involved'])}")
    print("=" * 60)


async def main():
    """Main execution"""
    print("\n🚀 MULTI-AGENT ENTERTAINMENT DISCOVERY SYSTEM")
    print("Solving the '45-minute decision problem'\n")

    # Create coordinator
    coordinator = CoordinatorAgent()

    # Execute workflow
    user_query = "I want to watch something highly rated tonight"
    result = await coordinator.execute(user_query)

    # Display results
    display_recommendations(result)

    print("\n💡 This demo shows 3 agents working together:")
    print("   1. ResearchAgent - Finds content across platforms")
    print("   2. AnalysisAgent - Analyzes and ranks options")
    print("   3. RecommendationAgent - Generates personalized picks")
    print("\n✨ All coordinated by the CoordinatorAgent!")


if __name__ == "__main__":
    asyncio.run(main())
