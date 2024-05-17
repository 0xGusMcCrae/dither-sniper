from telethon import TelegramClient, events
import requests
import os
from dotenv import load_dotenv
from datetime import datetime
import discord
from discord.ext import commands

from parse_message import parse_message

from logger_config import LoggerConfig

logger_config = LoggerConfig()
log = logger_config.get_logger()

load_dotenv()

api_id = int(os.getenv('API_ID'))
api_hash = os.getenv('API_HASH')
phone_number = os.getenv('PHONE_NUMBER')
group_id = int(os.getenv('GROUP_ID'))
group_access_hash = os.getenv('ACCESS_HASH')
discord_bot_token = os.getenv('DISCORD_BOT_TOKEN')
discord_channel_id = int(os.getenv('DISCORD_CHANNEL_ID'))


# Initialize the Telegram client
client = TelegramClient('dither_listener', api_id, api_hash)

client.start(phone_number)

# Use the bot's username
bot_username = 'DitherSeerBot' 

intents = discord.Intents.default()

discord_bot = commands.Bot(command_prefix='!', intents=intents)

@discord_bot.event
async def on_ready():
    log.info(f'Logged in as {discord_bot.user}')

async def send_to_discord(message):
    channel = discord_bot.get_channel(discord_channel_id)
    if channel:
        await channel.send(message)
    else:
        log.error(f'Could not find channel with ID {discord_channel_id}')


async def main():
    await client.start(phone_number)
    log.info("Client Created")

    # Create an input peer using the group ID and access hash
    from telethon.tl.types import InputPeerChannel
    peer = InputPeerChannel(channel_id=group_id, access_hash=group_access_hash)

    @client.on(events.NewMessage(chats=peer))
    async def handler(event):
        message = event.message.message
        if message:
            log.info(f'New message: {message}')
        # response = requests.post(rust_bot_endpoint, json={'message': message})
        # log.info(f'Response from Rust bot: {response.status_code} - {response.text}')
        await send_to_discord(message, indent=4)


    await client.run_until_disconnected()

client.loop.run_until_complete(main())
