# 🚀 Backend API Template (SQLx + Redis + Nginx)

Este é um template de arquitetura escalável e pronta para produção, focado em performance, segurança e alta disponibilidade.

## 🏗️ Arquitetura & Tecnologias

* **Linguagem/Framework:** Rust/ActixWeb
* **Banco de Dados:** PostgreSQL com **SQLx** para queries seguras e alta performance.
* **Cache:** Redis implementando o padrão **Cache-Aside** (Lazy Loading).
* **Autenticação:** JWT (JSON Web Tokens) com **RBAC** (Role-Based Access Control).
* **Infraestrutura:** Docker & Docker Compose.
* **Escalabilidade:** **Nginx** atuando como Reverse Proxy e **Load Balancer**.

---

## 🛠️ Pré-requisitos

* Docker e Docker Compose v2+
* Rust 1.84+

---

## 🚦 Como Iniciar

1.  **Clonar o repositório:**
    ```bash
    git clone [https://github.com/DFaltGP/rust-cache-aside-template.git](https://github.com/DFaltGP/rust-cache-aside-template.git)
    cd rust-cache-aside-template
    ```

2.  **Configurar Variáveis de Ambiente:**
    ```bash
    cp .env.example .env
    ```

3.  **Subir o ambiente (Docker):**
    ```bash
    docker-compose up -d --build
    ```
    Este comando levanta:
    * **PostgreSQL** (Porta 5432)
    * **Redis** (Porta 6379)
    * **API Nodes** (Escalados internamente)
    * **Nginx** (Porta 80/443)

---

## 🔒 Autenticação e Roles

O sistema utiliza controle de acesso baseado em níveis:
* **Admin:** Acesso total ao sistema.
* **Editor:** Pode criar e editar recursos.
* **Viewer:** Acesso apenas de leitura.

Os tokens JWT devem ser enviados no Header:
`Authorization: Bearer <seu_token>`

---

## 📂 Estrutura do Projeto

```text
├── .docker/              # Configurações de Dockerfile e Nginx (Load Balancer)
├── migrations/           # Scripts SQL de migração para o SQLx
├── src/
│   ├── core/             # Núcleo da aplicação
│   │   ├── auth/         # Lógica de autenticação, handlers e models de login
│   │   └── middleware/   # Middlewares de JWT, Logging e RBAC
│   ├── main.rs           # Ponto de entrada da aplicação e rotas
│   └── utils.rs          # Funções utilitárias e helpers globais
├── .env.example          # Modelo de variáveis de ambiente
└── docker-compose.yml
```
  ## Performance e Resiliência

  **Cache-Aside:** Redução de carga no banco de dados através de invalidação inteligente no Redis.

  **Load Balancing:** O Nginx distribui o tráfego entre os containers da API, garantindo que o sistema continue online mesmo se uma instância falhar.

  **Conexões:** Pool de conexões otimizado via SQLx.
