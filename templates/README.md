# Investment Wallet

Uma carteira de investimentos em Rust com atualização automática de preços de tokens Solana e gráfico de evolução do valor total.

## Funcionalidades

### ✅ Autenticação de Usuários
- Cadastro e login com JWT
- Senhas armazenadas com hash bcrypt
- Sessões gerenciadas via cookies

### ✅ Gestão de Investimentos
- Adicionar novos ativos com endereço do token Solana
- Listar todos os investimentos do usuário
- Remover ativos da carteira
- Cálculo automático do valor total da carteira

### ✅ Atualização Automática de Preços
- Integração com a API DexScreener para buscar preços em tempo real
- Tarefa em background que atualiza os preços a cada 5 minutos
- Suporte a qualquer token Solana listado em DEXs

### ✅ Histórico e Gráfico de Evolução
- Registro automático do valor total da carteira no banco de dados
- Gráfico interativo com Chart.js que mostra a evolução do valor
- Cálculo e exibição do percentual de valorização total
- Visualização de toda a trajetória da carteira desde o primeiro registro

## Tecnologias Utilizadas

### Backend
- **Rust** com Axum (framework web)
- **Tokio** para runtime assíncrono
- **SQLx** para comunicação com MySQL
- **JWT** para autenticação
- **Bcrypt** para hash de senhas
- **Reqwest** para requisições HTTP à API DexScreener
- **Chrono** para manipulação de datas
- **Askama** para templates HTML

### Frontend
- **HTML5** com Tailwind CSS para estilização
- **Chart.js** para gráficos interativos
- JavaScript vanilla para interações

### Banco de Dados
- **MySQL** com as tabelas:
  - `users` (dados dos usuários)
  - `investments` (investimentos de cada usuário)
  - `portfolio_history` (histórico de valor da carteira)

## Como Executar

### Pré-requisitos
- Rust e Cargo instalados
- MySQL server rodando
- Conta no GitHub (para clonar o repositório)

### Passos
1. Clone o repositório:
   ```bash
   git clone https://github.com/FabianoBeze/investment_wallet.git
   cd investment_wallet


### Estrutura do projeto

   investment_wallet/
├── src/
│   ├── main.rs           # Ponto de entrada da aplicação
│   ├── handlers.rs       # Handlers das rotas HTTP
│   ├── models.rs         # Structs de dados (User, Investment, PortfolioHistoryEntry)
│   ├── repository.rs     # Métodos de acesso ao banco de dados
│   └── services/
│       └── dex_screener.rs # Integração com a API DexScreener
├── templates/
│   ├── index.html        # Template principal da carteira
│   ├── login.html        # Página de login
│   └── register.html     # Página de cadastro
├── .env                  # Variáveis de ambiente (já configurado)
├── Cargo.toml            # Dependências do Rust
└── README.md             # Este arquivo

Autor
Fabiano Beze

Licença
MIT
