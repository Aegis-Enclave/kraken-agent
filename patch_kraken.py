import re
import sys

def main():
    file_path = '/home/lxrdxe7o/.hermes/hermes-agent/hermes_cli/skin_engine.py'
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Find the start of the kraken block
    start_match = re.search(r'    "kraken": \{.*?"name": "kraken",', content, re.DOTALL)
    if not start_match:
        print("Could not find kraken skin definition.")
        sys.exit(1)
        
    start_idx = start_match.start()
    
    end_match = re.search(r'\n\}\n\n\n# =============================================================================\n# Skin loading', content, re.DOTALL)
    if not end_match:
        print("Could not find end of _BUILTIN_SKINS.")
        sys.exit(1)
        
    end_idx = end_match.start()
    
    new_kraken = r'''    "kraken": {
        "name": "kraken",
        "description": "Deep-sea Monokai \u2014 purple, green, and neon from the abyss",
        "colors": {
            "banner_border": "#AE81FF",
            "banner_title": "#F8F8F2",
            "banner_accent": "#A6E22E",
            "banner_dim": "#75715E",
            "banner_text": "#F8F8F2",
            "ui_accent": "#66D9EF",
            "ui_label": "#E6DB74",
            "ui_ok": "#A6E22E",
            "ui_error": "#F92672",
            "ui_warn": "#FD971F",
            "prompt": "#66D9EF",
            "input_rule": "#75715E",
            "response_border": "#AE81FF",
            "status_bar_bg": "#1B1D1E",
            "status_bar_text": "#F8F8F2",
            "status_bar_strong": "#E6DB74",
            "status_bar_dim": "#75715E",
            "status_bar_good": "#A6E22E",
            "status_bar_warn": "#FD971F",
            "status_bar_bad": "#F92672",
            "status_bar_critical": "#F92672",
            "status_bar_yolo": "#FD971F",
            "session_label": "#E6DB74",
            "session_border": "#75715E",
            "voice_status_bg": "#1B1D1E",
            "completion_menu_bg": "#1B1D1E",
            "completion_menu_current_bg": "#3E3D32",
            "completion_menu_meta_bg": "#272822",
            "completion_menu_meta_current_bg": "#49483E",
        },
        "spinner": {
            "waiting_faces": ["(\u2261)", "(\u2248)", "(\u223f)", "(\u25c8)", "(\u03a8)", "(\U0001f991)"],
            "thinking_faces": ["(\u2261)", "(\u2248)", "(\u223f)", "(\u03a8)", "(\u25c8)", "(\u2301)", "(\u223e)"],
            "thinking_verbs": [
                "stirring the abyss", "unfurling tentacles", "charting deep currents",
                "inking the void", "tangling with the unknown", "sounding the trench",
                "coiling for strike", "reading pressure ridges", "glowing in the dark",
                "shedding a bioluminescent tear", "befriending a gulper eel",
                "counting jellyfish", "spiraling downward", "mapping the sea floor",
                "teasing the leviathan", "whispering to barnacles", "summoning the maelstrom",
                "harvesting black pearls", "wrestling submarines", "breaching the surface"
            ],
            "wings": [
                ["\u27ea\u2261", "\u2261\u27eb"],
                ["\u27ea\u2248", "\u2248\u27eb"],
                ["\u27ea\u223f", "\u223f\u27eb"],
                ["\u27ea\u25c8", "\u25c8\u27eb"],
                ["\u27ea\u03a8", "\u03a8\u27eb"],
            ],
        },
        "tool_emojis": {
            "run_shell_command": "\U0001f41a",
            "read_file": "\U0001f4dc",
            "write_file": "\U0001f58b\ufe0f",
            "replace": "\U0001f991",
            "grep_search": "\U0001f50e",
            "glob": "\U0001f30d",
            "ask_user": "\u2753",
            "enter_plan_mode": "\U0001f9ed",
            "invoke_agent": "\U0001f419",
            "activate_skill": "\u26a1"
        },
        "branding": {
            "agent_name": "Kraken Agent",
            "welcome": "The Kraken awakens. Type your command or /help to sound the depths.",
            "goodbye": "Returning to the abyss... \U0001f991",
            "response_label": " \u2261 Kraken ",
            "prompt_symbol": "\u223f",
            "help_header": "(\u2261) Abyssal Commands",
        },
        "tool_prefix": "\u223f",
        "banner_logo": """[bold #A6E22E] _  _______            _  ________ _   _            _____ ______ _   _ _______ [/]\n[bold #A6E22E]| |/ /  __ \\     /\\   | |/ /  ____| \\ | |     /\\   / ____|  ____| \\ | |__   __|[/]\n[bold #E6DB74]| ' /| |__) |   /  \\  | ' /| |__  |  \\| |    /  \\ | |  __| |__  |  \\| |  | |   [/]\n[bold #E6DB74]|  < |  _  /   / /\\ \\ |  < |  __| | . ` |   / /\\ \\| | |_ |  __| | . ` |  | |   [/]\n[bold #AE81FF]| . \\| | \\ \\  / ____ \\| . \\| |____| |\\  |  / ____ \\ |__| | |____| |\\  |  | |   [/]\n[bold #AE81FF]|_|\\_\\_|  \\_\\/_/    \\_\\_|\\_\\______|_| \\_| /_/    \\_\\_____|______|_| \\_|  |_|   [/]""",
        "banner_hero": """[#A6E22E]⣴⣶⣤⡤⠦⣤⣀⣤⠆     ⣈⣭⣿⣶⣿⣦⣼⣆[/]
[#F8F8F2] ⠉⠻⢿⣿⠿⣿⣿⣶⣦⠤⠄⡠⢾⣿⣿⡿⠋⠉⠉⠻⣿⣿⡛⣦[/]
[#AE81FF]      ⠈⢿⣿⣟⠦ ⣾⣿⣿⣷    ⠻⠿⢿⣿⣧⣄[/]
[#66D9EF]      ⣸⣿⣿⢧ ⢻⠻⣿⣿⣷⣄⣀⠄⠢⣀⡀⠈⠙⠿⠄[/]
[#E6DB74]     ⢠⣿⣿⣿⠈    ⣻⣿⣿⣿⣿⣿⣿⣿⣛⣳⣤⣀⣀[/]
[#FD971F] ⢠⣧⣶⣥⡤⢄ ⣸⣿⣿⠘  ⢀⣴⣿⣿⡿⠛⣿⣿⣧⠈⢿⠿⠟⠛⠻⠿⠄[/]
[#FD971F]⣰⣿⣿⠛⠻⣿⣿⡦⢹⣿⣷   ⢊⣿⣿⡏  ⢸⣿⣿⡇ ⢀⣠⣄⣾⠄[/]
[#F92672] ⣠⣿⠿⠛ ⢀⣿⣿⣷⠘⢿⣿⣦⡀ ⢸⢿⣿⣿⣄ ⣸⣿⣿⡇⣪⣿⡿⠿⣿⣷⡄[/]
[#F92672] ⠙⠃   ⣼⣿⡟  ⠈⠻⣿⣿⣦⣌⡇⠻⣿⣿⣷⣿⣿⣿ ⣿⣿⡇ ⠛⠻⢷⣄[/]
[#AE81FF]    ⢻⣿⣿⣄   ⠈⠻⣿⣿⣿⣷⣿⣿⣿⣿⣿⡟ ⠫⢿⣿⡆[/]
[dim #75715E]     ⠻⣿⣿⣿⣿⣶⣶⣾⣿⣿⣿⣿⣿⣿⣿⣿⡟⢀⣀⣤⣾⡿⠃[/]
[dim #75715E]                  from the abyss[/]"""'''

    new_content = content[:start_idx] + new_kraken + content[end_idx:]
    
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(new_content)

if __name__ == '__main__':
    main()