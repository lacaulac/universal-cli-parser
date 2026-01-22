def main():
    # Liste des comportements avec leurs numéros correspondants
    behaviour_map = {
        '0': 'NEUTRAL',
        '1': 'CMD_EXEC',
        '2': 'FILE_READ',
        '3': 'FILE_WRITE',
        '4': 'COPY',
        '5': 'NET_COMS',
        '6': 'NET_CFG',
        '7': 'NET_INFO',
        '8': 'SYS_INFO',
        '9': 'FS_INFO',
        '10': 'FS_OP',
        '11': 'ARG_FILE'
    }

    # Afficher la liste des comportements avec leurs numéros
    print("Behaviour options:")
    for num, behaviour in behaviour_map.items():
        print(f"{num}: {behaviour}")

    # Ask for header information
    name = input("Program name: ")
    has_separatorless_args_for_char_options = input("has_separatorless_args_for_char_options (true/false) [default: false]: ").lower() or 'false'
    string_separators = list(input("Enter string separators: "))
    handle_quotes = input("handle_quotes (true/false) [default: false]: ").lower() or 'false'

    # Demander les comportements pour l'en-tête
    behaviours_input = input("Enter behaviours (separated by spaces, use numbers): ").split()
    behaviours = [behaviour_map.get(num, '') for num in behaviours_input if num in behaviour_map]

    # Initialize lists for options
    char_options = []
    string_options = []

    while True:
        print("\nMenu:")
        print("1. Add a char option")
        print("2. Add a string option")
        print("3. Finish and generate the TOML file")
        choice = input("Choose an option (1/2/3): ")

        if choice == '1':
            option_name = input("Enter the char option name: ")
            has_arg = input("Does this option take an argument? (true/false) [default: false]: ").lower() or 'false'
            behaviours_input = input("Enter behaviours (separated by spaces, use numbers): ").split()
            behaviours = [behaviour_map.get(num, '') for num in behaviours_input if num in behaviour_map]
            char_options.append({
                "option_name": option_name,
                "has_arg": has_arg == 'true',
                "behaviours": behaviours
            })
        elif choice == '2':
            option_name = input("Enter the string option name: ")
            has_arg = input("Does this option take an argument? (true/false) [default: false]: ").lower() or 'false'
            behaviours_input = input("Enter behaviours (separated by spaces, use numbers): ").split()
            behaviours = [behaviour_map.get(num, '') for num in behaviours_input if num in behaviour_map]
            string_options.append({
                "option_name": option_name,
                "has_arg": has_arg == 'true',
                "behaviours": behaviours
            })
        elif choice == '3':
            break
        else:
            print("Invalid choice. Please choose 1, 2, or 3.")

    # Generate the TOML file
    with open("config.toml", "w") as f:
        f.write(f'name = "{name}"\n')
        f.write(f'has_separatorless_args_for_char_options = {has_separatorless_args_for_char_options}\n')
        f.write(f'string_separators = {string_separators}\n')
        f.write(f'handle_quotes = {handle_quotes}\n')
        f.write(f'behaviours = {behaviours}\n')
        f.write("\n")

        for char_option in char_options:
            f.write("[[char_options]]\n")
            f.write(f'option_name = "{char_option["option_name"]}"\n')
            f.write(f'has_arg = {char_option["has_arg"]}\n')
            f.write(f'behaviours = {char_option["behaviours"]}\n')
            f.write("\n")

        for string_option in string_options:
            f.write("[[string_options]]\n")
            f.write(f'option_name = "{string_option["option_name"]}"\n')
            f.write(f'has_arg = {string_option["has_arg"]}\n')
            f.write(f'behaviours = {string_option["behaviours"]}\n')
            f.write("\n")

    print("TOML file generated successfully!")

if __name__ == "__main__":
    main()
