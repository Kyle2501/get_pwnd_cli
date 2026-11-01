import os
import json
import asyncio
import tempfile
import discord
from discord import app_commands
from discord.ext import commands

class PPTXProcessorCog(commands.Cog):
    def __init__(self, bot):
        self.bot = bot
        # Path to your compiled Rust binary in your Termux project workspace
        self.rust_binary_path = os.getenv("PPTX_RUST_BIN", "./target/release/pptx_json_batch")

    @app_commands.command(
        name="process_deck",
        description="Process a PowerPoint presentation using Rust JSON substitution rules."
    )
    @app_commands.describe(
        file="The .pptx file to process",
        rules_json="JSON string containing key-value replacement rules"
    )
    async def process_deck(
        self, 
        interaction: discord.Interaction, 
        file: discord.Attachment, 
        rules_json: str
    ):
        await interaction.response.defer(thinking=True)

        # Validate file extension
        if not file.filename.lower().endswith(".pptx"):
            await interaction.followup.send("❌ Error: Attached file must be a valid `.pptx` presentation.", ephemeral=True)
            return

        # Validate JSON rules format
        try:
            rules_dict = json.loads(rules_json)
        except json.JSONDecodeError as e:
            await interaction.followup.send(f"❌ Error: Invalid JSON rules payload provided:\n`{e}`", ephemeral=True)
            return

        # Use a secure temporary directory for local file processing
        with tempfile.TemporaryDirectory() as tmpdir:
            input_dir = os.path.join(tmpdir, "input")
            output_dir = os.path.join(tmpdir, "output")
            rules_path = os.path.join(tmpdir, "rules.json")

            os.makedirs(input_dir, exist_ok=True)
            os.makedirs(output_dir, exist_ok=True)

            # Save incoming Discord attachment locally
            input_file_path = os.path.join(input_dir, file.filename)
            await file.save(input_file_path)

            # Write JSON rules file for the Rust CLI
            with open(rules_path, "w", encoding="utf-8") as f:
                json.dump(rules_dict, f, indent=2)

            # Construct Rust CLI subprocess command
            cmd = [
                self.rust_binary_path,
                "--input-dir", input_dir,
                "--output-dir", output_dir,
                "--rules", rules_path
            ]

            try:
                # Execute Rust binary asynchronously without blocking the event loop
                process = await asyncio.create_subprocess_exec(
                    *cmd,
                    stdout=asyncio.subprocess.PIPE,
                    stderr=asyncio.subprocess.PIPE
                )
                stdout, stderr = await process.communicate()

                if process.returncode != 0:
                    error_msg = stderr.decode().strip() or stdout.decode().strip()
                    await interaction.followup.send(f"❌ Rust CLI execution failed:\n```bash\n{error_msg}\n```")
                    return

                output_file_path = os.path.join(output_dir, file.filename)
                if not os.path.exists(output_file_path):
                    await interaction.followup.send("❌ Error: Rust binary completed, but output file was not generated.")
                    return

                # Send processed file back to Discord
                processed_file = discord.File(output_file_path, filename=f"processed_{file.filename}")
                await interaction.followup.send(
                    content=f"✅ Successfully processed **{file.filename}** using Rust backend engine!",
                    file=processed_file
                )

            except Exception as e:
                await interaction.followup.send(f"❌ Internal processing error: `{str(e)}`")

async def setup(bot):
    await bot.add_cog(PPTXProcessorCog(bot))
