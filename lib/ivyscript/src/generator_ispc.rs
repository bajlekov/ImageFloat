/*
  Copyright (C) 2011-2020 G. Bajlekov

    Ivy is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Ivy is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use crate::ast::{
    BinaryExpr, BinaryOp, Cond, Expr, Index, Literal, Prop, Stmt, UnaryExpr, UnaryOp,
};
use crate::function_id::function_id;

use crate::inference::{Inference, VarType};

pub struct Generator<'a> {
    ast: Vec<Stmt>,
    inference: RefCell<Inference<'a>>,
    constants: RefCell<HashMap<String, &'a Expr>>,
    functions: RefCell<HashMap<String, &'a Stmt>>,
    kernels: RefCell<HashMap<String, &'a Stmt>>,
    generated_constants: RefCell<Option<String>>,
    generated_functions: RefCell<HashMap<String, (String, String, VarType)>>, // collect specialized functions: (declaration, definition, return value)
    generated_kernels: RefCell<HashMap<String, String>>, // collect specialized kernels: (kernel)
    used_functions: RefCell<HashSet<String>>,
    temp: RefCell<String>,
}

impl<'a> Generator<'a> {
    #[allow(clippy::ptr_arg)]
    pub fn new(ast: Vec<Stmt>) -> Generator<'a> {
        Generator {
            ast,
            inference: RefCell::new(Inference::new()),
            constants: RefCell::new(HashMap::new()),
            functions: RefCell::new(HashMap::new()),
            kernels: RefCell::new(HashMap::new()),
            generated_constants: RefCell::new(None),
            generated_functions: RefCell::new(HashMap::new()),
            generated_kernels: RefCell::new(HashMap::new()),
            used_functions: RefCell::new(HashSet::new()),
            temp: RefCell::new(String::new()),
        }
    }

    pub fn prepare(&'a self) {
        for stmt in &self.ast {
            match stmt {
                Stmt::Const(id, expr) => {
                    self.constants.borrow_mut().insert(id.clone(), &expr);
                }
                Stmt::Function { id, args, .. } => {
                    let id = format!("___{}_{}", args.len(), id);
                    self.functions.borrow_mut().insert(id, &stmt);
                }
                Stmt::Kernel { id, .. } => {
                    self.kernels.borrow_mut().insert(id.clone(), &stmt);
                }
                _ => {}
            }
        }
    }

    fn function(&'a self, name: &str, input: &[VarType]) -> String {
        let id = function_id(name, input);
        let name = format!("___{}_{}", input.len(), name);

        if let Some((decl, def, _)) = self.generated_functions.borrow().get(&id) {
            if !self.used_functions.borrow().contains(&id) {
                let temp = self.temp.borrow().clone();
                let temp = format!("{}\n{}\n{}\n", decl, temp, def);
                self.temp.replace(temp);
                self.used_functions.borrow_mut().insert(id.clone());
            }

            return id;
        }

        let kernel_scope = self.inference.borrow().scope.current.get();
        self.inference.borrow().scope.open();
        self.inference.borrow().scope.set_parent(0);

        self.inference
            .borrow()
            .scope
            .add("return", VarType::Unknown);

        // parse function
        if let Some(Stmt::Function { args, body, .. }) = self.functions.borrow().get(&name) {
            let mut def = String::from("(varying int _x, varying int _y, varying int _z, \n");
            let mut decl;

            for (k, v) in args.iter().enumerate() {
                let arg = match input[k] {
                    VarType::Buffer { .. } => {
                        format!("uniform float {}[], uniform int ___str_{}[]", v, v)
                    }
                    VarType::Int => format!("int {}", v),
                    VarType::Float => format!("float {}", v),
                    VarType::Vec => format!("float<3> {}", v),
                    VarType::BoolArray(1, _, a, _, _, _) => format!("bool {}[{}]", v, a),
                    VarType::BoolArray(2, _, a, b, _, _) => format!("bool {}[{}][{}]", v, a, b),
                    VarType::BoolArray(3, _, a, b, c, _) => {
                        format!("bool {}[{}][{}][{}]", v, a, b, c)
                    }
                    VarType::BoolArray(4, _, a, b, c, d) => {
                        format!("bool {}[{}][{}][{}][{}]", v, a, b, c, d)
                    }
                    VarType::IntArray(1, _, a, _, _, _) => format!("int {}[{}]", v, a),
                    VarType::IntArray(2, _, a, b, _, _) => format!("int {}[{}][{}]", v, a, b),
                    VarType::IntArray(3, _, a, b, c, _) => {
                        format!("int {}[{}][{}][{}]", v, a, b, c)
                    }
                    VarType::IntArray(4, _, a, b, c, d) => {
                        format!("int {}[{}][{}][{}][{}]", v, a, b, c, d)
                    }
                    VarType::FloatArray(1, _, a, _, _, _) => format!("float {}[{}]", v, a),
                    VarType::FloatArray(2, _, a, b, _, _) => format!("float {}[{}][{}]", v, a, b),
                    VarType::FloatArray(3, _, a, b, c, _) => {
                        format!("float {}[{}][{}][{}]", v, a, b, c)
                    }
                    VarType::FloatArray(4, _, a, b, c, d) => {
                        format!("float {}[{}][{}][{}][{}]", v, a, b, c, d)
                    }
                    VarType::VecArray(1, _, a, _, _, _) => format!("float<3> {}[{}]", v, a),
                    VarType::VecArray(2, _, a, b, _, _) => format!("float<3> {}[{}][{}]", v, a, b),
                    VarType::VecArray(3, _, a, b, c, _) => {
                        format!("float<3> {}[{}][{}][{}]", v, a, b, c)
                    }
                    VarType::VecArray(4, _, a, b, c, d) => {
                        format!("float<3> {}[{}][{}][{}][{}]", v, a, b, c, d)
                    }
                    _ => String::from("/*** Error: Unknown type ***/"),
                };

                if k < args.len() - 1 {
                    def.push_str(&format!("\t{},\n", arg));
                } else {
                    def.push_str(&format!("\t{}\n", arg));
                }

                self.inference.borrow().scope.add(v, input[k]);
            }

            def.push_str(")");

            decl = def.clone();

            def.push_str(" {\n");

            for v in body {
                def.push_str(&self.gen_stmt(v));
            }

            let ret_type = self
                .inference
                .borrow()
                .scope
                .get("return")
                .unwrap_or(VarType::Unknown);

            def = format!(
                "{} {} {}}}",
                match ret_type {
                    VarType::Bool => "bool",
                    VarType::Int => "int",
                    VarType::Float => "float",
                    VarType::Vec => "float<3>",
                    VarType::Unknown => "void",
                    _ => "/*** Error: Unknown type ***/",
                },
                id,
                def
            );

            decl = format!(
                "{} {} {};",
                match ret_type {
                    VarType::Bool => "bool",
                    VarType::Int => "int",
                    VarType::Float => "float",
                    VarType::Vec => "float<3>",
                    VarType::Unknown => "void",
                    _ => "/*** Error: Unknown type ***/",
                },
                id,
                decl
            );

            let temp = self.temp.borrow().clone();
            let temp = format!("{}\n{}\n{}\n", &decl, temp, &def);
            self.temp.replace(temp);
            self.used_functions.borrow_mut().insert(id.clone());

            // TODO: register function return types in order to have them inferred properly
            // register generated_functions
            self.generated_functions
                .borrow_mut()
                .insert(id.clone(), (decl, def, ret_type));
        }

        self.inference.borrow().scope.close();
        self.inference.borrow().scope.set_current(kernel_scope);

        id
    }

    pub fn kernel(&'a self, name: &str, input: &[VarType]) -> Option<String> {
        self.inference.borrow().scope.clear();
        self.inference.borrow_mut().functions = Some(&self.generated_functions);

        if self.generated_constants.borrow().is_none() {
            let mut s = String::new();
            for (k, v) in self.constants.borrow().iter() {
                s.push_str(&format!("const {}", self.gen_var(k, v)));
            }
            self.generated_constants.replace(Some(s));
        }

        let id = function_id(name, input);
        if let Some(k) = self.generated_kernels.borrow().get(&id) {
            return Some(k.clone());
        }

        if let Some(Stmt::Kernel { id, args, body }) = self.kernels.borrow().get(name) {
            self.inference.borrow().scope.open();
            self.inference
                .borrow()
                .scope
                .add("return", VarType::Unknown);

            self.temp
                .replace(self.generated_constants.borrow().clone().unwrap());
            self.used_functions.borrow_mut().clear();

            let mut a = String::from("\n\tuniform int _dim[],\n");
            for (k, v) in args.iter().enumerate() {
                let arg = format!(
                    "{} {}{}",
                    match input[k] {
                        VarType::Buffer { .. } => "uniform float",
                        VarType::Int => "uniform int",
                        VarType::Float => "uniform float",
                        VarType::IntArray(1, ..) => "uniform int",
                        VarType::FloatArray(1, ..) => "uniform float",
                        _ => "/*** Error: Unknown type ***/",
                    },
                    v,
                    match input[k] {
                        VarType::Buffer { .. } => format!("[], uniform int ___str_{}[]", v),
                        VarType::IntArray(1, ..) => String::from("[]"),
                        VarType::FloatArray(1, ..) => String::from("[]"),
                        _ => String::from(""),
                    },
                );

                if k < args.len() - 1 {
                    a.push_str(&format!("\t{},\n", arg));
                } else {
                    a.push_str(&format!("\t{}\n", arg));
                }

                self.inference.borrow().scope.add(v, input[k]);
            }

            let mut s = format!("task void ___task_{} ({}) {{", id, &a);

            s.push_str(
                "
uniform int _xmin = _dim[0] + taskIndex0*_dim[6];
uniform int _xmax = _dim[0] + min(((uniform int)taskIndex0 + 1)*_dim[6], _dim[3]);
uniform int _ymin = _dim[1] + taskIndex1*_dim[7];
uniform int _ymax = _dim[1] + min(((uniform int)taskIndex1 + 1)*_dim[7], _dim[4]);
uniform int _zmin = _dim[2] + taskIndex2*_dim[8];
uniform int _zmax = _dim[2] + min(((uniform int)taskIndex2 + 1)*_dim[8], _dim[5]);

// swap _x___ and _y___ if _dim[3]==0
cif (_dim[3]<16 && _dim[4]>16) {
    uniform int _tmin = _ymin;
    uniform int _tmax = _ymax;
    _ymin = _xmin;
    _ymax = _xmax;
    _xmin = _tmin;
    _xmax = _tmax;
}

foreach (_z = _zmin ... _zmax, _1 = _ymin ... _ymax, _0 = _xmin ... _xmax) {

int _x, _y;
cif (_dim[3]<16 && _dim[4]>16) {
    _y = _0;
    _x = _1;   
} else {
    _x = _0;
    _y = _1;   
}\n",
            );

            for v in body {
                s.push_str(&self.gen_stmt(v));
            }

            if self.inference.borrow().scope.get("return") != Some(VarType::Unknown) {
                return None;
            }

            self.inference.borrow().scope.close();
            s.push_str("}\n}\n\n");

            s.push_str(&format!(
                "export void {} ({}) {{",
                function_id(id, input),
                &a
            ));

            s.push_str(
                "
uniform int _nx = ceil((uniform float)_dim[3]/_dim[6]);
uniform int _ny = ceil((uniform float)_dim[4]/_dim[7]);
uniform int _nz = ceil((uniform float)_dim[5]/_dim[8]);
",
            );
            s.push_str(&format!("launch[_nx, _ny, _nz] ___task_{}(", id));
            s.push_str("\n\t_dim,\n\t");

            for (k, v) in args.iter().enumerate() {
                match input[k] {
                    VarType::Buffer { .. } => s.push_str(&format!("{}, ___str_{}", v, v)),
                    _ => s.push_str(&format!("{}", v)),
                }
                if k < args.len() - 1 {
                    s.push_str(", ");
                } else {
                    s.push_str(");\n");
                }
            }

            s.push_str("}");

            return Some(format!(
                "#include \"cs.ispc\"\n{}\n{}",
                self.temp.borrow().clone(),
                s
            ));
        }

        None
    }

    fn gen_stmt(&'a self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::Var(id, expr) => self.gen_var(id, expr),
            Stmt::Const(id, expr) => format!("const {}", self.gen_var(id, expr)),
            Stmt::Assign(id, expr) => self.gen_assign(id, expr),
            Stmt::Call(id, args) => {
                let args_str = args
                    .iter()
                    .map(|e| self.gen_expr(e))
                    .collect::<Vec<String>>();
                let vars = args
                    .iter()
                    .map(|e| self.inference.borrow().var_type(e))
                    .collect::<Vec<VarType>>();
                if self.inference.borrow().builtin(id, args).is_some() {
                    format!("{};\n", self.gen_call(id, &args_str, &vars))
                } else {
                    let id = self.function(id, &vars);
                    let mut args_str = args_str;
                    let mut vars = vars;
                    args_str.insert(0, String::from("_x, _y, _z"));
                    vars.insert(0, VarType::Unknown);
                    format!("{};\n", self.gen_call(&id, &args_str, &vars))
                }
            }
            Stmt::For {
                var,
                from,
                to,
                step,
                body,
            } => self.gen_for(var, from, to, step, body),
            Stmt::IfElse {
                cond_list,
                else_body,
            } => self.gen_if_else(cond_list, else_body),
            Stmt::While { cond, body } => self.gen_while(cond, body),
            Stmt::Return(None) => {
                if let Some(VarType::Unknown) = self.inference.borrow().scope.get("return") {
                    String::from("return;\n")
                } else {
                    String::from("// ERROR!!!\n")
                }
            }
            Stmt::Continue => String::from("continue;\n"),
            Stmt::Break => String::from("break;\n"),
            Stmt::Return(Some(expr)) => {
                let expr_str = self.gen_expr(expr); // generate before assessing type!

                // return value is either new, same as--, or promoted from the previous one
                let t1 = self.inference.borrow().var_type(expr);
                let t2 = self
                    .inference
                    .borrow()
                    .scope
                    .get("return")
                    .unwrap_or(VarType::Unknown);
                let t3 = self.inference.borrow().promote(t1, t2);

                if t2 == VarType::Unknown {
                    self.inference.borrow().scope.overwrite("return", t1);
                } else if t3 == VarType::Unknown {
                    return String::from("// ERROR!!!\n");
                } else {
                    self.inference.borrow().scope.overwrite("return", t3);
                }

                format!("return {};\n", expr_str)
            }
            Stmt::Comment(c) => format!("//{}\n", c),
            _ => String::from("// ERROR!!!\n"),
        }
    }

    fn gen_for(
        &'a self,
        var: &str,
        from: &Expr,
        to: &Expr,
        step: &Option<Expr>,
        body: &[Stmt],
    ) -> String {
        self.inference.borrow().scope.open();

        let mut s;

        // infer var type
        let from_type = self.inference.borrow().var_type(from);
        let to_type = self.inference.borrow().var_type(to);

        let mut var_type = self.inference.borrow().promote_num(from_type, to_type);

        if let Some(step) = &step {
            let step_type = self.inference.borrow().var_type(step);
            var_type = self.inference.borrow().promote_num(var_type, step_type);
            self.inference.borrow().scope.add(var, var_type);

            s = format!(
                "for ({var_type} {var} = {from}; ({step}>0)?({var}<={to}):({var}>={to}); {var} += {step}) {{\n",
                var_type = match var_type {
                    VarType::Int => "int",
                    VarType::Float => "float",
                    _ => "/*** Error: Unknown type ***/",
                },
                var = var,
                from = self.gen_expr(&from),
                to = self.gen_expr(&to),
                step = self.gen_expr(&step),
            )
        } else {
            let step = match var_type {
                VarType::Int => Expr::Literal(Literal::Int(1)),
                VarType::Float => Expr::Literal(Literal::Float(1.0)),
                VarType::Vec => Expr::Call(
                    String::from("vec"),
                    vec![Expr::Literal(Literal::Float(1.0))],
                ),
                _ => return String::from("// ERROR!!!\n"),
            };
            self.inference.borrow().scope.add(var, var_type);

            s = format!(
                "for ({var_type} {var} = {from}; {var}<={to}; {var} += {step}) {{\n",
                var_type = match var_type {
                    VarType::Int => "int",
                    VarType::Float => "float",
                    VarType::Vec => "float<3>",
                    _ => "/*** Error: Unknown type ***/",
                },
                var = var,
                from = self.gen_expr(&from),
                to = self.gen_expr(&to),
                step = self.gen_expr(&step),
            )
        }

        for v in body {
            s.push_str(&self.gen_stmt(v));
        }

        s.push_str("}\n");
        self.inference.borrow().scope.close();
        s
    }

    fn gen_if_else(&'a self, cond_list: &[Cond], else_body: &[Stmt]) -> String {
        // cond_list should have 1 or more entries

        let Cond { ref cond, ref body } = cond_list[0];

        let mut s = format!("if ({}) {{\n", self.gen_expr(cond));
        assert!(self.inference.borrow().var_type(cond) == VarType::Bool); // type info available only after generation!

        self.inference.borrow().scope.open();
        for v in body {
            s.push_str(&self.gen_stmt(v));
        }
        self.inference.borrow().scope.close();

        for cond_item in cond_list.into_iter().skip(1) {
            let Cond { ref cond, ref body } = cond_item;

            s.push_str(&format!("}} else if ({}) {{\n", self.gen_expr(cond)));
            assert!(self.inference.borrow().var_type(cond) == VarType::Bool); // type info available only after generation!

            self.inference.borrow().scope.open();
            for v in body {
                s.push_str(&self.gen_stmt(v));
            }
            self.inference.borrow().scope.close();
        }

        if !else_body.is_empty() {
            s.push_str("} else {\n");
            self.inference.borrow().scope.open();
            for v in else_body {
                s.push_str(&self.gen_stmt(v));
            }
            self.inference.borrow().scope.close();
        }
        s.push_str("}\n");

        s
    }

    fn gen_while(&'a self, cond: &Expr, body: &[Stmt]) -> String {
        assert!(self.inference.borrow().var_type(cond) == VarType::Bool);

        let mut s = format!("while ({}) {{\n", self.gen_expr(cond));

        self.inference.borrow().scope.open();
        for v in body {
            s.push_str(&self.gen_stmt(v));
        }
        self.inference.borrow().scope.close();
        s.push_str("}}\n");

        s
    }

    fn gen_var(&'a self, id: &str, expr: &Expr) -> String {
        let expr_str = match expr {
            Expr::Call(f, _) => match f.as_ref() {
                "array" => String::new(),
                "bool_array" => String::new(),
                "int_array" => String::new(),
                "float_array" => String::new(),
                "vec_array" => String::new(),
                _ => self.gen_expr(&expr),
            },
            Expr::Array(_) => format!(" = {}", self.gen_expr(&expr)),
            _ => self.gen_expr(&expr),
        };

        let var_type = self.inference.borrow().var_type(expr);
        self.inference.borrow().scope.add(id, var_type);

        match var_type {
            VarType::Bool => format!("bool {} = {};\n", id, expr_str),
            VarType::Int => format!("int {} = {};\n", id, expr_str),
            VarType::Float => format!("float {} = {};\n", id, expr_str),
            VarType::Vec => format!("float<3> {} = {};\n", id, expr_str),

            VarType::BoolArray(1, _, a, _, _, _) => format!("bool {} [{}]{};\n", id, a, expr_str),
            VarType::BoolArray(2, _, a, b, _, _) => {
                format!("bool {} [{}][{}]{};\n", id, a, b, expr_str)
            }
            VarType::BoolArray(3, _, a, b, c, _) => {
                format!("bool {} [{}][{}][{}]{};\n", id, a, b, c, expr_str)
            }
            VarType::BoolArray(4, _, a, b, c, d) => {
                format!("bool {} [{}][{}][{}][{}]{};\n", id, a, b, c, d, expr_str)
            }

            VarType::IntArray(1, _, a, _, _, _) => format!("int {} [{}]{};\n", id, a, expr_str),
            VarType::IntArray(2, _, a, b, _, _) => {
                format!("int {} [{}][{}]{};\n", id, a, b, expr_str)
            }
            VarType::IntArray(3, _, a, b, c, _) => {
                format!("int {} [{}][{}][{}]{};\n", id, a, b, c, expr_str)
            }
            VarType::IntArray(4, _, a, b, c, d) => {
                format!("int {} [{}][{}][{}][{}]{};\n", id, a, b, c, d, expr_str)
            }

            VarType::FloatArray(1, _, a, _, _, _) => format!("float {} [{}]{};\n", id, a, expr_str),
            VarType::FloatArray(2, _, a, b, _, _) => {
                format!("float {} [{}][{}]{};\n", id, a, b, expr_str)
            }
            VarType::FloatArray(3, _, a, b, c, _) => {
                format!("float {} [{}][{}][{}]{};\n", id, a, b, c, expr_str)
            }
            VarType::FloatArray(4, _, a, b, c, d) => {
                format!("float {} [{}][{}][{}][{}]{};\n", id, a, b, c, d, expr_str)
            }

            VarType::VecArray(1, _, a, _, _, _) => {
                format!("float<3> {} [{}]{};\n", id, a, expr_str)
            }
            VarType::VecArray(2, _, a, b, _, _) => {
                format!("float<3> {} [{}][{}]{};\n", id, a, b, expr_str)
            }
            VarType::VecArray(3, _, a, b, c, _) => {
                format!("float<3> {} [{}][{}][{}]{};\n", id, a, b, c, expr_str)
            }
            VarType::VecArray(4, _, a, b, c, d) => format!(
                "float<3> {} [{}][{}][{}][{}]{};\n",
                id, a, b, c, d, expr_str
            ),

            _ => String::from("// ERROR!!!\n"),
        }
    }

    fn gen_expr(&'a self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(Literal::Bool(true)) => String::from("true"),
            Expr::Literal(Literal::Bool(false)) => String::from("false"),
            Expr::Literal(Literal::Int(n)) => format!("{}", n),
            Expr::Literal(Literal::Float(n)) => format!("{:.7}f", n),
            Expr::Unary(expr) => self.gen_unary(expr),
            Expr::Binary(expr) => self.gen_binary(expr),
            Expr::Identifier(id) => id.clone(),
            Expr::Index(expr, idx) => self.gen_index(expr, idx),
            Expr::Grouping(expr) => format!("({})", self.gen_expr(expr)),
            Expr::Call(id, args) => {
                if args.len() == 1 {
                    match (id.as_ref(), &args[0]) {
                        ("get_global_id", Expr::Literal(Literal::Int(0))) => {
                            return String::from("_x")
                        }
                        ("get_global_id", Expr::Literal(Literal::Int(1))) => {
                            return String::from("_y")
                        }
                        ("get_global_id", Expr::Literal(Literal::Int(2))) => {
                            return String::from("_z")
                        }
                        _ => {}
                    }
                }

                let args_str = args
                    .iter()
                    .map(|e| self.gen_expr(e))
                    .collect::<Vec<String>>();
                let vars = args
                    .iter()
                    .map(|e| self.inference.borrow().var_type(e))
                    .collect::<Vec<VarType>>();
                if self.inference.borrow().builtin(id, args).is_some() {
                    self.gen_call(id, &args_str, &vars)
                } else {
                    let id = self.function(id, &vars);
                    let mut args_str = args_str;
                    let mut vars = vars;
                    args_str.insert(0, String::from("_x, _y, _z"));
                    vars.insert(0, VarType::Unknown);
                    self.gen_call(&id, &args_str, &vars)
                }
            }
            Expr::Array(elems) => {
                let mut s = String::new();
                for (k, v) in elems.iter().enumerate() {
                    s.push_str(&self.gen_expr(v));
                    if k < elems.len() - 1 {
                        s.push_str(", ");
                    }
                }
                return format!("{{{}}}", s);
            }
            _ => String::from("// ERROR!!!\n"),
        }
    }

    fn gen_unary(&'a self, expr: &UnaryExpr) -> String {
        match expr.op {
            UnaryOp::Not => format!("!{}", self.gen_expr(&expr.right)),
            UnaryOp::Neg => format!("(-{})", self.gen_expr(&expr.right)),
        }
    }

    fn gen_binary(&'a self, expr: &BinaryExpr) -> String {
        match expr.op {
            BinaryOp::And => format!(
                "{} && {}",
                self.gen_expr(&expr.left),
                self.gen_expr(&expr.right)
            ),
            BinaryOp::Or => format!(
                "{} || {}",
                self.gen_expr(&expr.left),
                self.gen_expr(&expr.right)
            ),

            BinaryOp::Sub => format!(
                "{} - {}",
                self.gen_expr(&expr.left),
                self.gen_expr(&expr.right)
            ),
            BinaryOp::Add => format!(
                "{} + {}",
                self.gen_expr(&expr.left),
                self.gen_expr(&expr.right)
            ),
            BinaryOp::Div => {
                if self.inference.borrow().var_type(&expr.left) == VarType::Int {
                    format!(
                        "((float){})/{}",
                        self.gen_expr(&expr.left),
                        self.gen_expr(&expr.right),
                    )
                } else {
                    format!(
                        "{}/{}",
                        self.gen_expr(&expr.left),
                        self.gen_expr(&expr.right),
                    )
                }
            }
            BinaryOp::Mul => format!(
                "{}*{}",
                self.gen_expr(&expr.left),
                self.gen_expr(&expr.right)
            ),
            BinaryOp::Mod => format!(
                "{}%{}",
                self.gen_expr(&expr.left),
                self.gen_expr(&expr.right)
            ),
            BinaryOp::Pow => {
                let call = if self.inference.borrow().var_type(&expr.right) == VarType::Int {
                    "pown"
                } else {
                    "pow"
                };

                if self.inference.borrow().var_type(&expr.left) == VarType::Int {
                    format!(
                        "{}((float)({}), {})",
                        call,
                        self.gen_expr(&expr.left),
                        self.gen_expr(&expr.right)
                    )
                } else {
                    format!(
                        "{}({}, {})",
                        call,
                        self.gen_expr(&expr.left),
                        self.gen_expr(&expr.right)
                    )
                }
            }

            BinaryOp::Equal => format!(
                "{}=={}",
                self.gen_expr(&expr.left),
                self.gen_expr(&expr.right)
            ),
            BinaryOp::NotEqual => format!(
                "{}!={}",
                self.gen_expr(&expr.left),
                self.gen_expr(&expr.right)
            ),

            BinaryOp::Less => format!(
                "{}<{}",
                self.gen_expr(&expr.left),
                self.gen_expr(&expr.right)
            ),
            BinaryOp::LessEqual => format!(
                "{}<={}",
                self.gen_expr(&expr.left),
                self.gen_expr(&expr.right)
            ),
            BinaryOp::Greater => format!(
                "{}>{}",
                self.gen_expr(&expr.left),
                self.gen_expr(&expr.right)
            ),
            BinaryOp::GreaterEqual => format!(
                "{}>={}",
                self.gen_expr(&expr.left),
                self.gen_expr(&expr.right)
            ),
        }
    }

    fn gen_call(&'a self, id: &str, args: &[String], vars: &[VarType]) -> String {
        let mut id = match id {
            "bool" => "(bool)",
            "int" => "(int)",
            "float" => "(float)",
            "vec" => "vec",
            _ => id,
        };

        if !vars.is_empty() {
            id = match (id, vars[0]) {
                ("abs", VarType::Float) => "abs",
                ("abs", VarType::Vec) => "abs",
                ("atomic_add", VarType::FloatArray(1, ..)) => "_atomic_float_add",
                ("atomic_sub", VarType::FloatArray(1, ..)) => "_atomic_float_sub",
                ("atomic_inc", VarType::FloatArray(1, ..)) => "_atomic_float_inc",
                ("atomic_dec", VarType::FloatArray(1, ..)) => "_atomic_float_dec",
                ("atomic_min", VarType::FloatArray(1, ..)) => "_atomic_float_min",
                ("atomic_max", VarType::FloatArray(1, ..)) => "_atomic_float_max",
                _ => id,
            }
        }

        let mut s = String::new();
        for (k, v) in args.iter().enumerate() {
            s.push_str(&v);
            if let VarType::Buffer { .. } = vars[k] {
                s.push_str(", ___str_");
                s.push_str(&v);
            }

            if k < args.len() - 1 {
                s.push_str(", ");
            }
        }

        return format!("{}({})", id, s);
    }

    fn gen_assign(&'a self, expr: &Expr, val: &Expr) -> String {
        if let Expr::Index(expr, idx) = expr {
            if let Index::ColorSpace(cs_from) = &**idx {
                // assign vec with color space conversion
                if let Expr::Index(id, idx) = &**expr {
                    if let Expr::Identifier(name) = &**id {
                        if let Index::Array2D(a, b) = &**idx {
                            let var = self.inference.borrow().var_type(id);
                            if let VarType::Buffer { z, cs } = var {
                                let cs = format!("{}to{}", cs_from, cs);
                                let a = self.gen_expr(a);
                                let b = self.gen_expr(b);
                                let guard = format!(
                                    "cif ({}>=0 && {}<___str_{}[0] && {}>=0 && {}<___str_{}[1]) ",
                                    a, a, name, b, b, name
                                );
                                let val = self.gen_expr(val);
                                if z == 3 {
                                    let id_x = format!(
                                        "{}[(varying int)({})]",
                                        name,
                                        var.idx_3d(name, &a, &b, "0")
                                    );
                                    let id_y = format!(
                                        "{}[(varying int)({})]",
                                        name,
                                        var.idx_3d(name, &a, &b, "1")
                                    );
                                    let id_z = format!(
                                        "{}[(varying int)({})]",
                                        name,
                                        var.idx_3d(name, &a, &b, "2")
                                    );
                                    format!("{} {{ float<3> __v = {}({}); {} = __v.x; {} = __v.y; {} = __v.z; }}\n",
                                        guard, cs, val, id_x, id_y, id_z)
                                } else if z == 1 {
                                    // match buffer storage size to color space
                                    let id = format!(
                                        "{}[(varying int)({})]",
                                        name,
                                        var.idx_3d(name, &a, &b, "0")
                                    );
                                    format!("{} {} = {}({});\n", guard, id, cs, val)
                                } else {
                                    String::from("// ERROR!!!\n")
                                }
                            } else {
                                String::from("// ERROR!!!\n")
                            }
                        } else {
                            String::from("// ERROR!!!\n")
                        }
                    } else {
                        String::from("// ERROR!!!\n")
                    }
                } else {
                    String::from("// ERROR!!!\n")
                }
            } else if let Index::Array1D(a) = &**idx {
                let var = self.inference.borrow().var_type(expr);
                if let Expr::Identifier(name) = &**expr {
                    match var {
                        VarType::BoolArray(1, ..)
                        | VarType::IntArray(1, ..)
                        | VarType::FloatArray(1, ..)
                        | VarType::VecArray(1, ..) => {
                            format!("{}[{}] = {};\n", name, self.gen_expr(a), self.gen_expr(val))
                        }
                        VarType::Buffer { .. } => {
                            let a = self.gen_expr(a);
                            let val = self.gen_expr(val);
                            let guard = format!(
                                "cif ({}>=0 && {}<(___str_{}[0] * ___str_{}[1] * ___str_{}[2])) ",
                                a, a, name, name, name,
                            );

                            let id = format!("{}[(varying int)({})]", name, var.idx_1d(name, &a));
                            format!("{} {} = {};\n", guard, id, val)
                        }
                        _ => String::from("// ERROR!!!\n"),
                    }
                } else {
                    String::from("// ERROR!!!\n")
                }
            } else if let Index::Array2D(a, b) = &**idx {
                let var = self.inference.borrow().var_type(expr);
                if let Expr::Identifier(name) = &**expr {
                    match var {
                        VarType::Buffer { z: 1, .. } => {
                            let a = self.gen_expr(a);
                            let b = self.gen_expr(b);
                            let guard = format!(
                                "cif ({}>=0 && {}<___str_{}[0] && {}>=0 && {}<___str_{}[1]) ",
                                a, a, name, b, b, name
                            );
                            let val = self.gen_expr(val);

                            let id = format!(
                                "{}[(varying int)({})]",
                                name,
                                var.idx_3d(name, &a, &b, "0")
                            );
                            format!("{} {} = {};\n", guard, id, val)
                        }
                        VarType::Buffer { z: 3, .. } => {
                            let a = self.gen_expr(a);
                            let b = self.gen_expr(b);
                            let guard = format!(
                                "cif ({}>=0 && {}<___str_{}[0] && {}>=0 && {}<___str_{}[1]) ",
                                a, a, name, b, b, name
                            );
                            let val = self.gen_expr(val);

                            let id_x = format!(
                                "{}[(varying int)({})]",
                                name,
                                var.idx_3d(name, &a, &b, "0")
                            );
                            let id_y = format!(
                                "{}[(varying int)({})]",
                                name,
                                var.idx_3d(name, &a, &b, "1")
                            );
                            let id_z = format!(
                                "{}[(varying int)({})]",
                                name,
                                var.idx_3d(name, &a, &b, "2")
                            );
                            format!(
                                "{} {{ float<3> __v = {}; {} = __v.x; {} = __v.y; {} = __v.z; }}\n",
                                guard, val, id_x, id_y, id_z
                            )
                        }
                        VarType::BoolArray(2, ..)
                        | VarType::IntArray(2, ..)
                        | VarType::FloatArray(2, ..)
                        | VarType::VecArray(2, ..) => format!(
                            "{}[{}][{}] = {};\n",
                            name,
                            self.gen_expr(a),
                            self.gen_expr(b),
                            self.gen_expr(val)
                        ),
                        _ => String::from("// ERROR!!!\n"),
                    }
                } else {
                    String::from("// ERROR!!!\n")
                }
            } else if let Index::Array3D(a, b, c) = &**idx {
                let var = self.inference.borrow().var_type(expr);
                if let Expr::Identifier(name) = &**expr {
                    match var {
                        VarType::BoolArray(3, ..)
                        | VarType::IntArray(3, ..)
                        | VarType::FloatArray(3, ..)
                        | VarType::VecArray(3, ..) => format!(
                            "{}[{}][{}][{}] = {};\n",
                            name,
                            self.gen_expr(a),
                            self.gen_expr(b),
                            self.gen_expr(c),
                            self.gen_expr(val)
                        ),
                        VarType::Buffer { .. } => {
                            let a = self.gen_expr(a);
                            let b = self.gen_expr(b);
                            let c = self.gen_expr(c);
                            let guard = format!(
                                "cif ({}>=0 && {}<___str_{}[0] && {}>=0 && {}<___str_{}[1] && {}>=0 && {}<___str_{}[2]) ",
                                a, a, name, b, b, name, c, c, name
                            );
                            let val = self.gen_expr(val);

                            let id = format!(
                                "{}[(varying int)({})]",
                                name,
                                var.idx_3d(name, &a, &b, &c)
                            );
                            format!("{} {} = {};\n", guard, id, val)
                        }
                        _ => String::from("// ERROR!!!\n"),
                    }
                } else {
                    String::from("// ERROR!!!\n")
                }
            } else if let Index::Array4D(a, b, c, d) = &**idx {
                let var = self.inference.borrow().var_type(expr);
                if let Expr::Identifier(name) = &**expr {
                    match var {
                        VarType::BoolArray(4, ..)
                        | VarType::IntArray(4, ..)
                        | VarType::FloatArray(4, ..)
                        | VarType::VecArray(4, ..) => format!(
                            "{}[{}][{}][{}][{}] = {};\n",
                            name,
                            self.gen_expr(a),
                            self.gen_expr(b),
                            self.gen_expr(c),
                            self.gen_expr(d),
                            self.gen_expr(val)
                        ),
                        _ => String::from("// ERROR!!!\n"),
                    }
                } else {
                    String::from("// ERROR!!!\n")
                }
            } else {
                let id = self.gen_index(expr, idx);
                format!("{} = {};\n", id, self.gen_expr(val))
            }
        } else {
            format!("{} = {};\n", self.gen_expr(expr), self.gen_expr(val))
        }
    }

    fn gen_index(&'a self, expr: &Expr, idx: &Index) -> String {
        let name = if let Expr::Identifier(name) = expr {
            name
        } else {
            "/***ERROR***/"
        };
        match idx {
            Index::Vec(0) => {
                let var = self.inference.borrow().var_type(expr);
                match var {
                    VarType::Vec => format!("{}.x", self.gen_expr(expr)),
                    VarType::Buffer { .. } => format!("___str_{}[0]", name),
                    _ => String::from("// ERROR!!!\n"),
                }
            }
            Index::Vec(1) => {
                let var = self.inference.borrow().var_type(expr);
                match var {
                    VarType::Vec => format!("{}.y", self.gen_expr(expr)),
                    VarType::Buffer { .. } => format!("___str_{}[1]", name),
                    _ => String::from("// ERROR!!!\n"),
                }
            }
            Index::Vec(2) => {
                let var = self.inference.borrow().var_type(expr);
                match var {
                    VarType::Vec => format!("{}.z", self.gen_expr(expr)),
                    VarType::Buffer { .. } => format!("___str_{}[2]", name),
                    _ => String::from("// ERROR!!!\n"),
                }
            }
            Index::Array1D(a) => {
                if let Expr::Identifier(id) = expr {
                    let var = self.inference.borrow().var_type(expr);
                    match var {
                        VarType::Buffer { .. } => var.buf_idx_1d(id, &self.gen_expr(a)),
                        VarType::BoolArray(1, ..)
                        | VarType::IntArray(1, ..)
                        | VarType::FloatArray(1, ..)
                        | VarType::VecArray(1, ..) => format!("{}[{}]", id, self.gen_expr(a)),
                        _ => String::from("// ERROR!!!\n"),
                    }
                } else {
                    String::from("// ERROR!!!\n")
                }
            }
            Index::Array2D(a, b) => {
                if let Expr::Identifier(id) = expr {
                    let var = self.inference.borrow().var_type(expr);
                    match var {
                        VarType::Buffer { z: 1, .. } => {
                            var.buf_idx_3d(id, &self.gen_expr(a), &self.gen_expr(b), "0")
                        }
                        VarType::Buffer { z: 3, .. } => format!(
                            "vec{}",
                            var.buf_idx_2d(id, &self.gen_expr(a), &self.gen_expr(b))
                        ),
                        VarType::BoolArray(2, ..)
                        | VarType::IntArray(2, ..)
                        | VarType::FloatArray(2, ..)
                        | VarType::VecArray(2, ..) => {
                            format!("{}[{}][{}]", id, self.gen_expr(a), self.gen_expr(b))
                        }
                        _ => String::from("// ERROR!!!\n"),
                    }
                } else {
                    String::from("// ERROR!!!\n")
                }
            }
            Index::Array3D(a, b, c) => {
                if let Expr::Identifier(id) = expr {
                    let var = self.inference.borrow().var_type(expr);
                    match var {
                        VarType::Buffer { .. } => var.buf_idx_3d(
                            id,
                            &self.gen_expr(a),
                            &self.gen_expr(b),
                            &self.gen_expr(c),
                        ),
                        VarType::BoolArray(3, ..)
                        | VarType::IntArray(3, ..)
                        | VarType::FloatArray(3, ..)
                        | VarType::VecArray(3, ..) => format!(
                            "{}[{}][{}][{}]",
                            id,
                            self.gen_expr(a),
                            self.gen_expr(b),
                            self.gen_expr(c),
                        ),
                        _ => String::from("// ERROR!!!\n"),
                    }
                } else {
                    String::from("// ERROR!!!\n")
                }
            }
            Index::Array4D(a, b, c, d) => {
                if let Expr::Identifier(id) = expr {
                    let var = self.inference.borrow().var_type(expr);
                    match var {
                        VarType::BoolArray(4, ..)
                        | VarType::IntArray(4, ..)
                        | VarType::FloatArray(4, ..)
                        | VarType::VecArray(4, ..) => format!(
                            "{}[{}][{}][{}][{}]",
                            id,
                            self.gen_expr(a),
                            self.gen_expr(b),
                            self.gen_expr(c),
                            self.gen_expr(d),
                        ),
                        _ => String::from("// ERROR!!!\n"),
                    }
                } else {
                    String::from("// ERROR!!!\n")
                }
            }
            Index::ColorSpace(cs_to) => {
                if let Expr::Index(expr, idx) = expr {
                    if let Expr::Identifier(id) = &**expr {
                        let var = self.inference.borrow().var_type(expr);
                        if let VarType::Buffer { z, cs, .. } = var {
                            let id = if let Index::Array2D(a, b) = &**idx {
                                if z == 1 {
                                    var.buf_idx_3d(id, &self.gen_expr(a), &self.gen_expr(b), "0")
                                } else if z == 3 {
                                    format!(
                                        "vec{}",
                                        var.buf_idx_2d(id, &self.gen_expr(a), &self.gen_expr(b))
                                    )
                                } else {
                                    String::from("// ERROR!!!\n")
                                }
                            } else {
                                String::from("// ERROR!!!\n")
                            };
                            format!("{}to{}({})", cs, cs_to, id)
                        } else {
                            String::from("// ERROR!!!\n")
                        }
                    } else {
                        String::from("// ERROR!!!\n")
                    }
                } else {
                    String::from("// ERROR!!!\n")
                }
            }

            Index::Prop(prop) => {
                if let Expr::Index(expr, idx) = expr {
                    if let Expr::Identifier(id) = &**expr {
                        let var = self.inference.borrow().var_type(expr);
                        let idx = &**idx;
                        let idx = match (var, idx) {
                            (VarType::Buffer { .. }, Index::Array1D(a)) => {
                                var.idx_1d(id, &self.gen_expr(a))
                            }
                            (VarType::Buffer { z: 1, .. }, Index::Array2D(a, b)) => {
                                var.idx_3d(id, &self.gen_expr(a), &self.gen_expr(b), "0")
                            }
                            (VarType::Buffer { .. }, Index::Array3D(a, b, c)) => var.idx_3d(
                                id,
                                &self.gen_expr(a),
                                &self.gen_expr(b),
                                &self.gen_expr(c),
                            ),
                            _ => String::from("// ERROR!!!\n"),
                        };
                        match prop {
                            Prop::Int => format!("(((uniform int*){})[{}])", id, idx),
                            Prop::Idx => idx,
                            Prop::Ptr => format!("({} + {})", id, idx),
                            Prop::IntPtr => format!("(((uniform int*){}) + {})", id, idx),
                        }
                    } else {
                        String::from("// ERROR!!!\n")
                    }
                } else {
                    String::from("// ERROR!!!\n")
                }
            }
            _ => String::from("// ERROR!!!\n"),
        }
    }
}
