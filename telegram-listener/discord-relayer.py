import os
import json
from flask import Flask, request, jsonify
from discord.ext import commands
from dotenv import load_dotenv
import asyncio
import discord

load_dotenv()

bot_username = 'boop' 
discord_bot_token = os.getenv('DISCORD_BOT_TOKEN')
discord_channel_id = int(os.getenv('DISCORD_CHANNEL_ID'))

intents = discord.Intents.default()
intents.messages = True
discord_bot = commands.Bot(command_prefix='!', intents=intents)

app = Flask(__name__)

@discord_bot.event
async def on_ready():
    print(f'Logged in as {discord_bot.user} (ID: {discord_bot.user.id})')
    for guild in discord_bot.guilds:
        print(f'Connected to server: {guild.name} (ID: {guild.id})')
        for channel in guild.text_channels:
            print(f'Available channel: {channel.name} (ID: {channel.id})')

async def send_to_discord(message):
    channel = discord_bot.get_channel(discord_channel_id)
    if channel:
        await channel.send(message)
    else:
        print(f'Could not find channel with ID {discord_channel_id}')

@app.route('/telegram', methods=['POST'])
def receive_from_telegram():
    data = request.data.decode('utf-8')
    if data:
        asyncio.run_coroutine_threadsafe(send_to_discord(data), discord_bot.loop)
    return jsonify({'status': 'success'}), 200

def run_flask_app():
    app.run(host='0.0.0.0', port=8000)

if __name__ == "__main__":
    import threading
    threading.Thread(target=run_flask_app).start()
    discord_bot.run(discord_bot_token)
