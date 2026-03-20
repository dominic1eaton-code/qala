import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-book',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './book.component.html',
  styleUrl: './book.component.css'
})
export class SolutionsBookComponent {}
