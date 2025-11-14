"""
Discord Bot for Steam Party Picker (Phase 3)
Allows users to create sessions and get recommendations directly in Discord
"""

import os
import discord
from discord.ext import commands
from discord import app_commands
import aiohttp
from typing import Optional
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Bot configuration
DISCORD_TOKEN = os.getenv("DISCORD_BOT_TOKEN")
API_URL = os.getenv("BACKEND_URL", "http://backend:8000/api")

# Initialize bot
intents = discord.Intents.default()
intents.message_content = True
bot = commands.Bot(command_prefix="!", intents=intents)
tree = bot.tree


@bot.event
async def on_ready():
    """Bot startup event"""
    logger.info(f"Bot logged in as {bot.user}")
    try:
        synced = await tree.sync()
        logger.info(f"Synced {len(synced)} command(s)")
    except Exception as e:
        logger.error(f"Failed to sync commands: {e}")


@tree.command(name="gameparty", description="Create a new game session")
@app_commands.describe(name="Optional session name")
async def create_session(interaction: discord.Interaction, name: Optional[str] = None):
    """Create a new game recommendation session"""
    await interaction.response.defer()

    try:
        async with aiohttp.ClientSession() as session:
            async with session.post(
                f"{API_URL}/sessions/",
                json={"name": name or f"{interaction.user.name}'s session"}
            ) as response:
                if response.status == 201:
                    data = await response.json()
                    session_code = data["session_code"]

                    embed = discord.Embed(
                        title="🎮 Game Session Created!",
                        description=f"Session code: **{session_code}**",
                        color=discord.Color.green()
                    )
                    embed.add_field(
                        name="Join Link",
                        value=f"https://steampartypicker.com/session/{session_code}",
                        inline=False
                    )
                    embed.add_field(
                        name="How to join",
                        value="Click the link above and connect your Steam profile",
                        inline=False
                    )

                    await interaction.followup.send(embed=embed)
                else:
                    await interaction.followup.send("❌ Failed to create session")

    except Exception as e:
        logger.error(f"Error creating session: {e}")
        await interaction.followup.send("❌ An error occurred")


@tree.command(name="recommend", description="Get game recommendations for a session")
@app_commands.describe(session_code="The session code")
async def get_recommendations(interaction: discord.Interaction, session_code: str):
    """Get recommendations for a session"""
    await interaction.response.defer()

    try:
        async with aiohttp.ClientSession() as session:
            # Generate recommendations
            async with session.post(
                f"{API_URL}/recommendations/{session_code}/generate",
                json={"count": 5}
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    recommendations = data["recommendations"]

                    if not recommendations:
                        await interaction.followup.send("No recommendations found")
                        return

                    embed = discord.Embed(
                        title=f"🎮 Game Recommendations",
                        description=f"Top {len(recommendations)} games for your group",
                        color=discord.Color.blue()
                    )

                    for rec in recommendations[:5]:
                        game = rec["game"]
                        category_emoji = {
                            "G1": "🟢",
                            "G2": "🟠",
                            "G3": "🟣"
                        }
                        emoji = category_emoji.get(rec["ownership_category"], "⚪")

                        field_value = f"{emoji} {rec['ownership_category']}"
                        if rec.get("explanation"):
                            field_value += f"\n{rec['explanation'][:100]}..."

                        embed.add_field(
                            name=f"#{rec['rank']} {game['title']}",
                            value=field_value,
                            inline=False
                        )

                    await interaction.followup.send(embed=embed)
                else:
                    await interaction.followup.send("❌ Failed to get recommendations")

    except Exception as e:
        logger.error(f"Error getting recommendations: {e}")
        await interaction.followup.send("❌ An error occurred")


@tree.command(name="session", description="Get session info")
@app_commands.describe(session_code="The session code")
async def session_info(interaction: discord.Interaction, session_code: str):
    """Get information about a session"""
    await interaction.response.defer()

    try:
        async with aiohttp.ClientSession() as session:
            async with session.get(f"{API_URL}/sessions/{session_code}") as response:
                if response.status == 200:
                    data = await response.json()

                    embed = discord.Embed(
                        title=f"📊 Session: {data.get('name', session_code)}",
                        description=f"Code: **{session_code}**",
                        color=discord.Color.purple()
                    )
                    embed.add_field(
                        name="Status",
                        value=data["status"].capitalize(),
                        inline=True
                    )
                    embed.add_field(
                        name="Participants",
                        value=str(len(data["participants"])),
                        inline=True
                    )

                    if data["participants"]:
                        participants_list = "\n".join([
                            f"• {p['nickname']}"
                            for p in data["participants"]
                        ])
                        embed.add_field(
                            name="Players",
                            value=participants_list,
                            inline=False
                        )

                    await interaction.followup.send(embed=embed)
                else:
                    await interaction.followup.send("❌ Session not found")

    except Exception as e:
        logger.error(f"Error getting session info: {e}")
        await interaction.followup.send("❌ An error occurred")


def run_bot():
    """Run the Discord bot"""
    if not DISCORD_TOKEN:
        logger.error("DISCORD_BOT_TOKEN not set")
        return

    bot.run(DISCORD_TOKEN)


if __name__ == "__main__":
    run_bot()
