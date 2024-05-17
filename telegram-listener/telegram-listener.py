from telethon import TelegramClient, events
import requests
import os
from dotenv import load_dotenv
from datetime import datetime
import asyncio
from telethon.tl.types import InputPeerChannel

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
http_endpoint = os.getenv('HTTP_ENDPOINT')

client = TelegramClient('dither_listener', api_id, api_hash)
peer = InputPeerChannel(channel_id=group_id, access_hash=group_access_hash)

@client.on(events.NewMessage(chats=peer))
async def handler(event):
    message = event.message.message
    parsed_message = parse_message(message)
    if parsed_message:  # still sending the unparsed message here, just want to use the parser to filter spam
        log.info(f'New message: {message}')
        try:
            response = requests.post(http_endpoint, data=message.encode('utf-8'), headers={'Content-Type': 'text/plain'})
            response.raise_for_status()
        except requests.exceptions.RequestException as e:
            print(f'Failed to send message to Discord bot: {e}')

async def main():
    await client.start(phone_number)
    log.info("Client Created")
    await client.run_until_disconnected()

    
if __name__ == "__main__":
    asyncio.run(main())


