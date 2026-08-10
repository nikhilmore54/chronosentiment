import sys

def main():
    file_path = "services/ultracrew_server/src/main.rs"
    with open(file_path, "r") as f:
        content = f.read()

    # Find and remove the disruption_recovery_handler function
    import re
    content = re.sub(r'async fn disruption_recovery_handler.*?Ok\(Json\(result\)\)\n}\n', '', content, flags=re.DOTALL)
    
    # Remove from router
    content = content.replace('\n            .route("/api/disruption_recovery", post(disruption_recovery_handler))', '')
    
    with open(file_path, "w") as f:
        f.write(content)

if __name__ == "__main__":
    main()
