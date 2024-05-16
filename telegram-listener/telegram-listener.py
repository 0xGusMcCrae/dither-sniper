from telethon import TelegramClient, events
import requests
import os
from dotenv import load_dotenv

load_dotenv()

api_id = int(os.getenv('API_ID'))
api_hash = os.getenv('API_HASH')
phone_number = os.getenv('PHONE_NUMBER')
group_id = int(os.getenv('GROUP_ID'))
group_access_hash = os.getenv('ACCESS_HASH')


# Initialize the Telegram client
client = TelegramClient('dither_listener', api_id, api_hash)

client.start(phone_number)

# Use the bot's username
bot_username = 'DitherSeerBot' 

async def main():
    await client.start(phone_number)
    print("Client Created")

    # Create an input peer using the group ID and access hash
    from telethon.tl.types import InputPeerChannel
    peer = InputPeerChannel(channel_id=group_id, access_hash=group_access_hash)

    @client.on(events.NewMessage(chats=peer))
    async def handler(event):
        message = event.message.message
        print(f'New message: {message}')
        # response = requests.post(rust_bot_endpoint, json={'message': message})
        # print(f'Response from Rust bot: {response.status_code} - {response.text}')


    await client.run_until_disconnected()

client.loop.run_until_complete(main())
