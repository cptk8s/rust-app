import requests
import logging
import sys
from datetime import datetime

# Configuración de logs
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[logging.StreamHandler(sys.stdout)]
)
logger = logging.getLogger(__name__)

BASE_URL = "http://localhost:3000"
USER_DATA = {
    "nombre": "Mutambo",
    "apellidos": "Williams",
    "direccion": "Casa de Africa 23",
    "telefono": "914314219"
}

def run_test():
    session = requests.Session()

    # 1. Login para obtener el token JWT
    logger.info("Paso 1: Intentando login...")
    login_payload = {"usuario": "admin", "clave": "d3d1c4fc3Aa"}
    response = session.post(f"{BASE_URL}/login", json=login_payload)
    
    if response.status_code != 200:
        logger.error(f"Fallo en Login: {response.status_code} - {response.text}")
        return

    token = response.json().get("token")
    logger.info(f"Token obtenido con éxito: {token[:20]}...")
    session.headers.update({"Authorization": f"Bearer {token}"})

    # 2. Obtener usuarios y verificar si existe Mutambo Williams
    logger.info("Paso 2: Obteniendo lista de usuarios...")
    response = session.get(f"{BASE_URL}/users")
    if response.status_code != 200:
        logger.error(f"Error obteniendo usuarios: {response.text}")
        return

    usuarios = response.json()
    mutambo = next((u for u in usuarios if u['nombre'] == "Mutambo" and u['apellidos'] == "Williams"), None)

    if not mutambo:
        logger.info("Mutambo Williams no encontrado. Procediendo a crearlo...")
        response = session.post(f"{BASE_URL}/users", json=USER_DATA)
        if response.status_code != 201:
            logger.error(f"Error creando usuario: {response.text}")
            return
        mutambo = response.json()
        logger.info(f"Usuario creado con ID: {mutambo['id']}")
    else:
        logger.info(f"Mutambo ya existía con ID: {mutambo['id']}")

    # 3. Verificar que ahora existe en la lista
    logger.info("Paso 3: Verificando persistencia del usuario...")
    response = session.get(f"{BASE_URL}/users")
    if not any(u['id'] == mutambo['id'] for u in response.json()):
        logger.error("Error: El usuario creado no aparece en la lista.")
        return
    logger.info("Verificación de usuario exitosa.")

    # 4. Crear una comunicación para Mutambo
    logger.info(f"Paso 4: Creando comunicación para usuario ID {mutambo['id']}...")
    com_payload = {
        "tipo": "Correo",
        "usuario_id": mutambo['id'],
        "resumen": "escribe solo para comprobar la comunicacion"
    }
    response = session.post(f"{BASE_URL}/comunicaciones", json=com_payload)
    if response.status_code != 201:
        logger.error(f"Error creando comunicación: {response.text}")
        return
    comunicacion = response.json()
    logger.info(f"Comunicación creada con ID: {comunicacion['id']}")

    # 5. Pedir comunicaciones y comprobar existencia y fecha
    logger.info("Paso 5: Verificando la comunicación y la fecha...")
    response = session.get(f"{BASE_URL}/comunicaciones")
    coms = response.json()
    
    # Buscamos la comunicación recién creada
    nueva_com = next((c for c in coms if c['id'] == comunicacion['id']), None)
    
    if nueva_com:
        fecha_hoy = datetime.now().strftime("%Y-%m-%d")
        if fecha_hoy in nueva_com['fecha']:
            logger.info(f"Comunicación confirmada para hoy: {nueva_com['fecha']}")
        else:
            logger.warning(f"La fecha {nueva_com['fecha']} no coincide con hoy {fecha_hoy}")
    else:
        logger.error("No se encontró la comunicación creada.")
        return

    # 6. Eliminar la comunicación
    logger.info(f"Paso 6: Eliminando comunicación ID {comunicacion['id']}...")
    response = session.delete(f"{BASE_URL}/comunicaciones/{comunicacion['id']}")
    if response.status_code != 204:
        logger.error(f"Error eliminando comunicación: {response.status_code}")
        return
    logger.info("Comunicación eliminada correctamente.")

    # 7. Eliminar el usuario
    logger.info(f"Paso 7: Eliminando usuario Mutambo ID {mutambo['id']}...")
    response = session.delete(f"{BASE_URL}/users/{mutambo['id']}")
    if response.status_code != 204:
        logger.error(f"Error eliminando usuario: {response.status_code}")
        return
    logger.info("Usuario eliminado correctamente. Test finalizado con éxito.")

if __name__ == "__main__":
    try:
        run_test()
    except requests.exceptions.ConnectionError:
        logger.error("No se pudo conectar con el API. ¿Está el contenedor corriendo en el puerto 3000?")