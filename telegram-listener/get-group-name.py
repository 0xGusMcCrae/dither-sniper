from telethon import TelegramClient, sync
import os
from dotenv import load_dotenv

load_dotenv()

# Set up your API ID, API Hash, and phone number
api_id = os.getenv("API_ID")
api_hash = os.getenv("API_HASH")
phone_number = os.getenv("PHONE_NUMBER")

# Initialize the Telegram client
client = TelegramClient('session_name', api_id, api_hash)

client.start(phone_number)

# Replace with part of the group name or exact group name
group_name = 'Dither AI'

# Fetch all dialogs (chats, channels, groups)
dialogs = client.get_dialogs()

for dialog in dialogs:
    if group_name.lower() in dialog.name.lower():
        print(f'Group Name: {dialog.name}')
        print(f'Group ID: {dialog.entity.id}')
        print(f'Group Access Hash: {dialog.entity.access_hash}')
        break

client.disconnect()
