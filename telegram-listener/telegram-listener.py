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


client = TelegramClient('dither_listener', api_id, api_hash)

client.start(phone_number)

bot_username = 'boop' 

intents = discord.Intents.default()
intents.messages = True
intents.guilds = True

discord_bot = commands.Bot(command_prefix='!', intents=intents)

@discord_bot.event
async def on_ready():
    log.info(f'Logged in as {discord_bot.user} (ID: {discord_bot.user.id})')
    await send_to_discord("hi friends")


async def send_to_discord(message):
    channel = discord_bot.get_channel(discord_channel_id)
    if channel:
        await channel.send(message)
    else:
        log.error(f'Could not find channel with ID {discord_channel_id}')


async def main():
    await client.start(phone_number)
    log.info("Client Created")

    from telethon.tl.types import InputPeerChannel
    peer = InputPeerChannel(channel_id=group_id, access_hash=group_access_hash)

    @client.on(events.NewMessage(chats=peer))
    async def handler(event):
        message = event.message.message
        parsed_message = parse_message(message)
        if parsed_message:  # still sending the unparsed message here, just want to use the parser to filter spam
            log.info(f'New message: {message}')
        await send_to_discord(message)


    await client.run_until_disconnected()

client.loop.run_until_complete(main())
discord_bot.run(discord_bot_token)
