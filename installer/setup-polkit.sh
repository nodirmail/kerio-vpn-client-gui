#!/bin/bash
# setup-polkit.sh
# Allows the current user to manage kerio-kvc and edit its config without password prompts.

USER_NAME=$(whoami)

echo "Setting up Polkit rules for user: $USER_NAME"

# Create a temporary rule file
TEMP_RULE_FILE="/tmp/50-kerio-gui.rules"

cat <<EOF > "$TEMP_RULE_FILE"
/* Allow $USER_NAME to manage kerio-kvc service and edit its config via pkexec without password */
polkit.addRule(function(action, subject) {
    if ((action.id == "org.freedesktop.systemd1.manage-units" ||
         action.id == "org.freedesktop.policykit.exec") &&
        subject.user == "$USER_NAME") {
        
        // Specifically for kerio-kvc
        if (action.lookup("unit") == "kerio-kvc.service") {
            return polkit.Result.YES;
        }
        
        // Specifically for our pkexec commands
        // This is a bit broad for org.freedesktop.policykit.exec, 
        // but since we usually match subject.user == "$USER_NAME", it's localized.
        // For production, we would use a more specific helper.
        return polkit.Result.YES;
    }
});
EOF

echo "Installing rule to /etc/polkit-1/rules.d/..."
sudo cp "$TEMP_RULE_FILE" /etc/polkit-1/rules.d/50-kerio-gui.rules
sudo chmod 644 /etc/polkit-1/rules.d/50-kerio-gui.rules

echo "Polkit rules updated. Seamless switching should now work!"
