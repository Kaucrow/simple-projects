# Google Images Scraper
# Tested on python 3.11.0b4
# Kaucrow - 2024

from selenium import webdriver
import os
import sys
import requests
import getopt
from selenium.webdriver.remote.webelement import WebElement
from selenium.webdriver.common.action_chains import ActionChains
from selenium.webdriver.common.by import By
from selenium.webdriver.chrome.service import Service
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC

WEBPAGE = None
DEBUG = False

argv = sys.argv[1:]

try:
    opts, args = getopt.getopt(argv, 'dw:', ['debug', 'webpage='])
except getopt.GetoptError as e:
    print(str(e))
    exit(2)

for opt, arg in opts:
    if opt in ['-w', '--webpage']:
        if arg.lower() in ['google', 'yandex']:
            WEBPAGE = arg.lower()
        else:
            print(arg.lower())
            print('[ ERR ]: The image webpage specified is not known.')
            exit(1)
    if opt in ['-d', '--debug']:
        DEBUG = True

if WEBPAGE == None:
    print('[ ERR ]: Must specify image webpage (google/yandex). E.g.: python main.py -w google')
    exit(1)

# Set up the web driver (make sure to download the appropriate driver for your browser)
options = webdriver.ChromeOptions()
if DEBUG == False:
    options.add_argument('--headless')
    options.add_argument('--no-sandbox')
    options.add_argument('--disable-gpu')
    options.add_argument('--disable-web-security')
    options.add_argument('--allow-running-insecure-content')
    options.add_argument('--allow-cross-origin-auth-prompt')

service = webdriver.ChromeService(executable_path='chromedriver.exe')
driver = webdriver.Chrome(options, service=service)
action = ActionChains(driver)

if WEBPAGE == 'google':
    def scrape_images(query, num_images, save_path):
        # Create a Google Images search URL
        search_url = f"https://www.google.com/search?q={query}&tbm=isch"

        # Open the Google Images search page
        driver.get(search_url)

        # Scroll down to load more images
        for _ in range(num_images // 50):
            driver.execute_script("window.scrollBy(0,10000)")

        # Wait for the images to load
        WebDriverWait(driver, 10).until(EC.presence_of_element_located((By.CSS_SELECTOR, "img.YQ4gaf")))

        # Get image elements (exclude the small ones at the top bar)
        img_elements = driver.find_elements(By.XPATH, "//img[contains(@class, 'YQ4gaf') and number(@width) >= 100 and number(@height) >= 100]")

        # Create the save directory
        os.makedirs(save_path, exist_ok=True)

        # Loop through the first num_images images
        for i, img_element in enumerate(img_elements[:num_images]):
            try:
                # Wait for the image to be clickable
                WebDriverWait(driver, 15).until(EC.element_to_be_clickable(img_element))

                # Click on each image to open it
                action.move_to_element(img_element).click().perform()

                # Wait for the opened image to load
                WebDriverWait(driver, 10).until(EC.presence_of_element_located((By.CSS_SELECTOR, 'img.sFlh5c.pT0Scc.iPVvYb')))

                # Get the URL of the opened image
                img_url_element = driver.find_element(By.CSS_SELECTOR, 'img.sFlh5c.pT0Scc.iPVvYb')
                img_url = img_url_element.get_attribute("src")

                ## Download the image
                img_name = f"{query}_{i+1}.jpg"
                img_path = os.path.join(save_path, img_name)
                response = requests.get(img_url, stream=True)
                with open(img_path, 'wb') as file:
                    for chunk in response.iter_content(chunk_size=1024):
                        if chunk:
                            file.write(chunk)
                print(f"Image {i+1} downloaded successfully")

            except Exception as e:
                print(f"Failed to download image {i+1}: {e}")

elif WEBPAGE == 'yandex':
    def scrape_images(query, num_images, save_path):
        # Create a Google Images search URL
        search_url = f"https://yandex.com/images/search?text={query}"

        # Open the Google Images search page
        driver.get(search_url)

        # Scroll down to load more images
        for _ in range(num_images // 50):
            driver.execute_script("window.scrollBy(0,10000)")

        # Wait for the images to load
        WebDriverWait(driver, 10).until(EC.presence_of_element_located((By.CSS_SELECTOR, "img.ContentImage-Image")))
        # Get image elements
        img_elements = driver.find_elements(By.CSS_SELECTOR, 'img.ContentImage-Image')

        # Create the save directory
        os.makedirs(save_path, exist_ok=True)

        # Loop through the first num_images images
        for i, img_element in enumerate(img_elements[:num_images]):
            try:
                # Ensure the image element is interactable
                WebDriverWait(driver, 20).until(EC.element_to_be_clickable(img_element))
                # Click on each image to open it
                action.move_to_element(img_element).perform()

                # Wait for the opened image to load
                WebDriverWait(driver, 10).until(EC.visibility_of_element_located((By.CSS_SELECTOR, 'a.Button2')))

                # Get the URL of the opened image
                img_url_element = driver.find_element(By.CLASS_NAME, 'SerpPolaroid-Download')
                img_url = img_url_element.get_attribute("href")
                print('--------------------------------------')
                print(img_url)
                print('--------------------------------------')

                ## Download the image
                img_name = f"{query}_{i+1}.jpg"
                img_path = os.path.join(save_path, img_name)
                response = requests.get(img_url, stream=True)
                with open(img_path, 'wb') as file:
                    for chunk in response.iter_content(chunk_size=1024):
                        if chunk:
                            file.write(chunk)
                print(f"Image {i+1} downloaded successfully")

            except Exception as e:
                print(f"Failed to download image {i+1}: {e}")

# Example usage
query = "dragons"
num_images = 20
save_path = "downloaded_images"
scrape_images(query, num_images, save_path)

# Close the browser
driver.quit()